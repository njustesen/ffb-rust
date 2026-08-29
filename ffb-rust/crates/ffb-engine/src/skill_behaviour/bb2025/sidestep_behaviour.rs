/// 1:1 translation of com.fumbbl.ffb.server.skillbehaviour.bb2025.SidestepBehaviour.
///
/// Priority 4 modifier on StepPushback.
use crate::model::skill_behaviour::SkillBehaviour as SbContainer;
use crate::model::step_modifier::StepModifierTrait;
use crate::step::framework::StepId;
use crate::skill_behaviour::registry::SkillRegistry;
use crate::step::bb2025::block::step_pushback::StepPushbackHookState;
use crate::util::util_server_pushback::UtilServerPushback;
use ffb_model::enums::{SkillId};
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::model::pushback_mode::PushbackMode;
use ffb_model::model::skill_use::SkillUse;
use ffb_model::report::report_skill_use::ReportSkillUse;
use ffb_model::types::FieldCoordinate;

// ── SidestepStepModifier ──────────────────────────────────────────────────────

// Java: When the defender has the Sidestep skill and conditions are met (free square, has tacklezone), the modifier prompts the defender to choose their own pushback square instead of the attacker deciding, switching pushback mode to SIDE_STEP and rebuilding available pushback squares accordingly.
pub struct SidestepStepModifier;

impl StepModifierTrait for SidestepStepModifier {
    fn applies_to(&self, step_id: StepId) -> bool { step_id == StepId::Pushback }

    fn priority(&self) -> i32 { 4 }

    /// Java: SidestepBehaviour.handleExecuteStepHook(StepPushback step, StepState state)
    ///
    /// Returns true if side-step was handled (either used or already decided),
    /// false if conditions not met.
    fn handle_execute_step(
        &self,
        game: &mut Game,
        _rng: &mut ffb_model::util::rng::GameRng,
        step_state: &mut dyn std::any::Any,
    ) -> bool {
        let state = step_state
            .downcast_mut::<StepPushbackHookState>()
            .expect("SidestepStepModifier: step_state must be StepPushbackHookState");

        let defender_id = state.defender_id.clone();

        // Java: UtilCards.hasSkill(state.defender, skill)
        let has_side_step = game.player(&defender_id)
            .map(|p| p.has_skill_property(NamedProperties::CAN_CHOOSE_OWN_PUSHED_BACK_SQUARE))
            .unwrap_or(false);

        // Java: Skill cancellingSkill = null;
        //       if (state.defender.getId().equals(game.getDefenderId()))
        //         cancellingSkill = UtilCards.getSkillCancelling(actingPlayer.getPlayer(), skill)
        let is_main_defender = game.defender_id.as_deref() == Some(&defender_id);
        let cancelling_skill = if is_main_defender {
            game.acting_player.player_id.as_deref()
                .and_then(|id| game.player(id))
                .and_then(|p| {
                    p.all_skill_ids().find(|id| {
                        id.properties().contains(&"cancelsCan ChooseOwnPushedBackSquare")
                            || id.properties().contains(&NamedProperties::CAN_PUSH_BACK_TO_ANY_SQUARE)
                    })
                })
        } else {
            None
        };

        // Java: boolean attackerHasConflictingSkill = cancellingSkill != null && cancellingSkill.conflictsWithAnySkill(actingPlayer.getPlayer())
        let attacker_cancels = game.acting_player.player_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| p.has_skill_property(NamedProperties::CAN_PUSH_BACK_TO_ANY_SQUARE))
            .unwrap_or(false);

        let defender_state = game.field_model.player_state(&defender_id);
        let has_tacklezones = defender_state.map(|s| s.has_tacklezones()).unwrap_or(true);
        let old_has_tacklezones = state.old_defender_state.map(|s| s.has_tacklezones()).unwrap_or(true);

        // Java: in_tacklezone condition
        let in_tacklezone = if state.pushback_stack_len == 0 {
            old_has_tacklezones
        } else {
            has_tacklezones
        };

        // Java: if (state.sideStepping.getOrDefault(id, true) && state.freeSquareAroundDefender
        //          && UtilCards.hasSkill(state.defender, skill)
        //          && !(cancellingSkill != null && !attackerHasConflictingSkill)
        //          && in_tacklezone)
        let using_side_step_default = *state.side_stepping.get(&defender_id).unwrap_or(&true);

        if using_side_step_default && state.free_square_around_defender && has_side_step
            && !(cancelling_skill.is_some() && !attacker_cancels)
            && in_tacklezone
        {
            // Java: if (!sideStepping.containsKey(id)) showDialog(DialogSkillUseParameter(Sidestep))
            // and CONTINUE, waiting for the coach's answer; re-enters with the value populated.
            //
            // This used to auto-answer TRUE inline, on the reasoning that the parity harness always
            // uses the skill, so the round-trip was unobservable. That was true of the RANDOM
            // contract, which answers a SKILL_USE for free. The heuristic SCORES `SkillUse` and
            // spends two sampler draws on it, so a prompt Rust never emits desynchronises the two
            // RNG streams for the rest of the game. Sidestep is bb2025-only and, on the amazon
            // roster, is carried by exactly one player -- the star Estelle la Veneaux -- which is
            // why bb2025 sat at 0/100 while bb2016 and bb2020 moved.
            //
            // A hook cannot raise a prompt, so park the request; `StepPushback` turns it into one
            // and files the answer back into `side_stepping`.
            if !state.side_stepping.contains_key(&defender_id) {
                state.pending_skill_use = Some((defender_id.clone(), SkillId::Sidestep));
                return true;
            }

            // Java: if (state.sideStepping.get(id)) { switch to SIDE_STEP mode }
            if *state.side_stepping.get(&defender_id).unwrap_or(&false) {
                state.pushback_mode = PushbackMode::SIDE_STEP;

                // Java: fieldModel.remove non-selected squares; rebuild in SIDE_STEP mode
                let home_choice = game.home_playing;
                let is_home = game.team_home.player(&defender_id).is_some();
                // side_step gives the defender's team choice
                let side_step_home_choice = if is_home { !game.home_playing } else { game.home_playing };

                if let Some(starting_sq) = state.starting_pushback_square {
                    let new_squares = UtilServerPushback::find_pushback_squares_grab(
                        starting_sq,
                        &|c: FieldCoordinate| game.field_model.player_at(c).is_some(),
                        &|_c| true,
                        side_step_home_choice,
                    );
                    state.pushback_squares = new_squares;
                } else {
                    state.pushback_squares.clear();
                }
            }

            // Java: publishParameter(STARTING_PUSHBACK_SQUARE, null)
            state.starting_pushback_square = None;
            return true;
        }

        // Java: else if (hasSkill && no_tacklezone) addReport(NO_TACKLEZONE)
        if has_side_step && (
            (state.pushback_stack_len == 0 && state.old_defender_state.map(|s| !s.has_tacklezones()).unwrap_or(false))
            || (state.pushback_stack_len > 0 && !has_tacklezones)
        ) {
            game.report_list.add(ReportSkillUse::new(
                game.defender_id.clone(),
                SkillId::Sidestep,
                false,
                SkillUse::NO_TACKLEZONE,
            ));
        }

        false
    }
}

// ── SidestepBehaviour ─────────────────────────────────────────────────────────

/// Side Step: player may choose their own pushback square after a block.
pub struct SidestepBehaviour;

impl SidestepBehaviour {
    pub fn new() -> Self { Self }

    pub fn register_into(registry: &mut SkillRegistry) {
        let mut sb = SbContainer::new();
        sb.register_step_modifier(Box::new(SidestepStepModifier));
        registry.register(SkillId::Sidestep, sb);
    }
}

impl Default for SidestepBehaviour {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skill_behaviour::registry::SkillRegistry;
    use crate::step::bb2025::block::step_pushback::StepPushbackHookState;
    use crate::step::framework::test_team;
    use ffb_model::enums::{PlayerState, PS_STANDING, PS_PRONE, Rules, SkillId};
    use ffb_model::model::game::Game;
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::types::FieldCoordinate;
    use ffb_model::util::rng::GameRng;
    use std::collections::HashMap;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn player_with_skills(id: &str, skills: Vec<SkillId>) -> Player {
        Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "pos".into(),
            player_type: ffb_model::enums::PlayerType::Regular,
            gender: ffb_model::enums::PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: skills.into_iter().map(|s| SkillWithValue { skill_id: s, value: None }).collect(),
            ..Default::default()
        }
    }

    fn default_hook_state(defender_id: &str, free_square: bool) -> StepPushbackHookState {
        let mut hs = StepPushbackHookState::new(
            defender_id.into(),
            Some(PlayerState::new(PS_STANDING)),
            None, 0, free_square, vec![],
            HashMap::new(), HashMap::new(), None,
        );
        hs
    }

    #[test]
    fn register_into_adds_step_modifier() {
        let mut reg = SkillRegistry::empty();
        SidestepBehaviour::register_into(&mut reg);
        let sb = reg.get(SkillId::Sidestep).expect("Sidestep must be registered");
        assert_eq!(sb.get_step_modifiers().len(), 1);
    }

    #[test]
    fn step_modifier_applies_to_pushback_step() {
        let m = SidestepStepModifier;
        assert!(m.applies_to(StepId::Pushback));
    }

    #[test]
    fn step_modifier_does_not_apply_to_wrong_step() {
        let m = SidestepStepModifier;
        assert!(!m.applies_to(StepId::BlockRoll));
    }

    #[test]
    fn step_modifier_priority_is_four() {
        let m = SidestepStepModifier;
        assert_eq!(m.priority(), 4);
    }

    #[test]
    fn no_side_step_skill_returns_false() {
        let mut game = make_game();
        game.team_away.players.push(player_with_skills("def1", vec![]));
        game.defender_id = Some("def1".into());
        game.field_model.set_player_coordinate("def1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("def1", PlayerState::new(PS_STANDING));

        let m = SidestepStepModifier;
        let mut hs = default_hook_state("def1", true);
        let result = m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hs);
        assert!(!result);
    }

    #[test]
    fn side_step_with_no_free_square_returns_false() {
        let mut game = make_game();
        game.team_away.players.push(player_with_skills("def1", vec![SkillId::Sidestep]));
        game.defender_id = Some("def1".into());
        game.field_model.set_player_coordinate("def1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("def1", PlayerState::new(PS_STANDING));

        let m = SidestepStepModifier;
        // free_square_around_defender = false
        let mut hs = default_hook_state("def1", false);
        let result = m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hs);
        assert!(!result, "SideStep should not fire when no free squares around defender");
    }

    /// A Sidestep defender's FIRST (undecided) pushback must ASK, not decide.
    ///
    /// Java shows a `DialogSkillUseParameter` and waits. This used to auto-answer `true` inline,
    /// reasoning that the parity harness always uses the skill so the round-trip was unobservable
    /// -- true of the random contract, which answers for free, and false of the heuristic, which
    /// scores `SkillUse` and spends two sampler draws on it. A prompt one engine never emits
    /// cannot be answered, and the two RNG streams part for the rest of the game.
    #[test]
    fn an_undecided_sidestep_asks_instead_of_deciding() {
        use ffb_model::enums::Direction;
        use ffb_model::types::PushbackSquare;
        let mut game = make_game();
        game.team_away.players.push(player_with_skills("def1", vec![SkillId::Sidestep]));
        game.defender_id = Some("def1".into());
        game.field_model.set_player_coordinate("def1", FieldCoordinate::new(10, 7));
        game.field_model.set_player_state("def1", PlayerState::new(PS_STANDING));

        let m = SidestepStepModifier;
        let mut hs = default_hook_state("def1", true);
        hs.starting_pushback_square =
            Some(PushbackSquare::new(FieldCoordinate::new(10, 7), Direction::North, true));
        let result = m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hs);

        assert!(result, "Java returns true from the hook when it shows the dialog");
        assert_eq!(
            hs.pending_skill_use,
            Some(("def1".to_string(), SkillId::Sidestep)),
            "the offer must be parked for the step to raise"
        );
        assert!(
            !hs.side_stepping.contains_key("def1"),
            "nothing may be decided before the coach answers"
        );
        assert_ne!(
            hs.pushback_mode,
            PushbackMode::SIDE_STEP,
            "the mode switch belongs to the ACCEPT path, not to making the offer"
        );
    }

    /// ...and once the answer is in, the hook takes its other branch and switches the mode.
    #[test]
    fn an_accepted_sidestep_switches_the_pushback_mode() {
        use ffb_model::enums::Direction;
        use ffb_model::types::PushbackSquare;
        let mut game = make_game();
        game.team_away.players.push(player_with_skills("def1", vec![SkillId::Sidestep]));
        game.defender_id = Some("def1".into());
        game.field_model.set_player_coordinate("def1", FieldCoordinate::new(10, 7));
        game.field_model.set_player_state("def1", PlayerState::new(PS_STANDING));

        let m = SidestepStepModifier;
        let mut hs = default_hook_state("def1", true);
        hs.starting_pushback_square =
            Some(PushbackSquare::new(FieldCoordinate::new(10, 7), Direction::North, true));
        hs.side_stepping.insert("def1".to_string(), true);
        let result = m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hs);

        assert!(result, "side step handled (used) returns true");
        assert!(hs.pending_skill_use.is_none(), "an answered offer must not be re-asked");
        assert_eq!(hs.pushback_mode, PushbackMode::SIDE_STEP);
    }

    #[test]
    fn side_step_declined_in_map_returns_false() {
        // Java: state.sideStepping.getOrDefault(id, true) returns false when map has false
        // → outer if block is false → method returns false
        let mut game = make_game();
        game.team_away.players.push(player_with_skills("def1", vec![SkillId::Sidestep]));
        game.defender_id = Some("def1".into());
        game.field_model.set_player_coordinate("def1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("def1", PlayerState::new(PS_STANDING));

        let m = SidestepStepModifier;
        let mut hs = default_hook_state("def1", true);
        // Pre-set to declined (false) → getOrDefault returns false → outer if is false
        hs.side_stepping.insert("def1".into(), false);
        let result = m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hs);
        // Returns false (outer if-condition was false)
        assert!(!result);
        // Mode should still be REGULAR since not entered
        assert_eq!(hs.pushback_mode, PushbackMode::REGULAR);
    }

    #[test]
    fn side_step_accepted_clears_starting_square() {
        use ffb_model::enums::Direction;
        use ffb_model::types::PushbackSquare;

        let mut game = make_game();
        game.team_away.players.push(player_with_skills("def1", vec![SkillId::Sidestep]));
        game.defender_id = Some("def1".into());
        game.field_model.set_player_coordinate("def1", FieldCoordinate::new(10, 7));
        game.field_model.set_player_state("def1", PlayerState::new(PS_STANDING));

        let m = SidestepStepModifier;
        let mut hs = default_hook_state("def1", true);
        hs.side_stepping.insert("def1".into(), true);
        let starting_sq = PushbackSquare::new(FieldCoordinate::new(10, 7), Direction::North, true);
        hs.starting_pushback_square = Some(starting_sq);

        m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hs);
        // Starting square should be cleared after SideStep is used
        assert!(hs.starting_pushback_square.is_none(), "starting square should be cleared when SideStep used");
    }

    #[test]
    fn side_step_accepted_switches_to_side_step_mode() {
        use ffb_model::enums::Direction;
        use ffb_model::types::PushbackSquare;

        let mut game = make_game();
        game.team_away.players.push(player_with_skills("def1", vec![SkillId::Sidestep]));
        game.defender_id = Some("def1".into());
        game.field_model.set_player_coordinate("def1", FieldCoordinate::new(10, 7));
        game.field_model.set_player_state("def1", PlayerState::new(PS_STANDING));

        let m = SidestepStepModifier;
        let mut hs = default_hook_state("def1", true);
        hs.side_stepping.insert("def1".into(), true);
        let starting_sq = PushbackSquare::new(FieldCoordinate::new(10, 7), Direction::North, true);
        hs.starting_pushback_square = Some(starting_sq);

        m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hs);
        assert_eq!(hs.pushback_mode, PushbackMode::SIDE_STEP, "should switch to SIDE_STEP mode when accepted");
    }

    #[test]
    fn side_step_with_no_tacklezone_adds_report() {
        let mut game = make_game();
        game.team_away.players.push(player_with_skills("def1", vec![SkillId::Sidestep]));
        game.defender_id = Some("def1".into());
        game.field_model.set_player_coordinate("def1", FieldCoordinate::new(5, 5));
        // Prone = no tacklezones
        game.field_model.set_player_state("def1", PlayerState::new(PS_PRONE));

        let m = SidestepStepModifier;
        let mut hs = StepPushbackHookState::new(
            "def1".into(),
            Some(PlayerState::new(PS_PRONE)), // old state also prone → no TZ
            None, 0, true, vec![],
            HashMap::new(), HashMap::new(), None,
        );

        m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hs);
        // Should add NO_TACKLEZONE report
        assert!(game.report_list.has_report(ffb_model::report::report_id::ReportId::SKILL_USE));
    }

}
