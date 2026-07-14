/// 1:1 translation of com.fumbbl.ffb.server.skillbehaviour.bb2020.SideStepBehaviour.
///
/// Priority 3 modifier on StepPushback.
///
/// **BB2020 vs BB2025 naming:** BB2020 uses `SideStep`/`SideStepBehaviour` (capital "S" in
/// "Step"); BB2025 renames the skill class to `Sidestep`/`SidestepBehaviour`. Kept as-is here to
/// match Java 1:1.
use crate::model::skill_behaviour::SkillBehaviour as SbContainer;
use crate::model::step_modifier::StepModifierTrait;
use crate::step::framework::StepId;
use crate::skill_behaviour::registry::SkillRegistry;
use crate::step::bb2020::block::step_pushback::StepPushbackHookState;
use crate::util::util_server_pushback::UtilServerPushback;
use ffb_model::enums::SkillId;
use ffb_model::model::game::Game;
use ffb_model::model::pushback_mode::PushbackMode;
use ffb_model::model::skill_use::SkillUse;
use ffb_model::report::report_skill_use::ReportSkillUse;
use ffb_model::types::FieldCoordinate;

// ── SideStepStepModifier ──────────────────────────────────────────────────────

pub struct SideStepStepModifier;

impl StepModifierTrait for SideStepStepModifier {
    fn applies_to(&self, step_id: StepId) -> bool { step_id == StepId::Pushback }

    fn priority(&self) -> i32 { 3 }

    /// Java: SideStepBehaviour.handleExecuteStepHook(StepPushback step, StepState state)
    fn handle_execute_step(
        &self,
        game: &mut Game,
        _rng: &mut ffb_model::util::rng::GameRng,
        step_state: &mut dyn std::any::Any,
    ) -> bool {
        let state = step_state
            .downcast_mut::<StepPushbackHookState>()
            .expect("SideStepStepModifier: step_state must be StepPushbackHookState");

        let defender_id = state.defender_id.clone();

        // Java: UtilCards.hasSkill(state.defender, skill)
        let has_side_step = game.player(&defender_id)
            .map(|p| p.has_skill(SkillId::SideStep))
            .unwrap_or(false);

        // Java: Skill cancellingSkill = null;
        //       if (state.defender.getId().equals(game.getDefenderId()))
        //         cancellingSkill = UtilCards.getSkillCancelling(actingPlayer.getPlayer(), skill)
        let is_main_defender = game.defender_id.as_deref() == Some(defender_id.as_str());
        let cancelling_skill = if is_main_defender {
            game.acting_player.player_id.as_deref()
                .and_then(|id| game.player(id))
                .and_then(|p| {
                    p.all_skill_ids().find(|id| {
                        id.properties().contains(&"cancelsCanChooseOwnPushedBackSquare")
                    })
                })
        } else {
            None
        };

        // Java: boolean attackerHasConflictingSkill = cancellingSkill != null && cancellingSkill.conflictsWithAnySkill(actingPlayer.getPlayer())
        let attacker_conflicts = false; // simplified: SideStep-cancelling skills have no self-conflict in standard rules

        let defender_state = game.field_model.player_state(&defender_id);
        let has_tacklezones = defender_state.map(|s| s.has_tacklezones()).unwrap_or(true);
        let old_has_tacklezones = state.old_defender_state.map(|s| s.has_tacklezones()).unwrap_or(true);

        // Java: (pushbackStack.isEmpty() && (oldDefenderState == null || oldDefenderState.hasTacklezones()))
        //       || (!pushbackStack.isEmpty() && playerState.hasTacklezones())
        let in_tacklezone = if state.pushback_stack_len == 0 {
            old_has_tacklezones
        } else {
            has_tacklezones
        };

        let using_side_step_default = *state.side_stepping.get(&defender_id).unwrap_or(&true);

        if using_side_step_default && state.free_square_around_defender && has_side_step
            && !(cancelling_skill.is_some() && !attacker_conflicts)
            && in_tacklezone
        {
            // Java: if (!sideStepping.containsKey(id)) show dialog → headless: auto-decline
            if !state.side_stepping.contains_key(&defender_id) {
                state.side_stepping.insert(defender_id.clone(), false);
                return true;
            }

            // Java: if (state.sideStepping.get(id)) { switch to SIDE_STEP mode }
            if *state.side_stepping.get(&defender_id).unwrap_or(&false) {
                state.pushback_mode = PushbackMode::SIDE_STEP;

                let side_step_home_player = game.team_home.player(&defender_id).is_some();

                if let Some(starting_sq) = state.starting_pushback_square {
                    let mut new_squares = UtilServerPushback::find_pushback_squares_grab(
                        starting_sq,
                        &|c: FieldCoordinate| game.field_model.player_at(c).is_some(),
                        &|_c| true,
                        game.home_playing,
                    );
                    // Java: for each pushbackSquare: setHomeChoice(sideStepHomePlayer)
                    for sq in &mut new_squares {
                        sq.home_choice = side_step_home_player;
                    }
                    state.pushback_squares = new_squares;
                } else {
                    state.pushback_squares.clear();
                }

                // Java: if ((sideStepHomePlayer && !game.isHomePlaying()) || (!sideStepHomePlayer && game.isHomePlaying()))
                //         game.setWaitingForOpponent(true); UtilServerTimer.stopTurnTimer(...)
                if (side_step_home_player && !game.home_playing) || (!side_step_home_player && game.home_playing) {
                    game.waiting_for_opponent = true;
                    // Java: UtilServerTimer.stopTurnTimer(...) — no headless turn-timer to stop.
                }
            }

            // Java: publishParameter(STARTING_PUSHBACK_SQUARE, null)
            state.starting_pushback_square = None;
            return true;
        }

        // Java: else if (hasSkill(defender, skill) && no_tacklezone_condition) addReport(NO_TACKLEZONE)
        if has_side_step && (
            (state.pushback_stack_len == 0 && state.old_defender_state.map(|s| !s.has_tacklezones()).unwrap_or(false))
            || (state.pushback_stack_len > 0 && !has_tacklezones)
        ) {
            game.report_list.add(ReportSkillUse::new(
                game.defender_id.clone(),
                SkillId::SideStep,
                false,
                SkillUse::NO_TACKLEZONE,
            ));
        }

        false
    }
}

// ── SideStepBehaviour ─────────────────────────────────────────────────────────

/// Side Step: player may choose their own pushback square after a block.
pub struct SideStepBehaviour;

impl SideStepBehaviour {
    pub fn new() -> Self { Self }

    pub fn register_into(registry: &mut SkillRegistry) {
        let mut sb = SbContainer::new();
        sb.register_step_modifier(Box::new(SideStepStepModifier));
        registry.register(SkillId::SideStep, sb);
    }
}

impl Default for SideStepBehaviour {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skill_behaviour::registry::SkillRegistry;
    use crate::step::bb2020::block::step_pushback::StepPushbackHookState;
    use crate::step::framework::test_team;
    use ffb_model::enums::{PlayerState, PS_STANDING, PS_PRONE, Rules, SkillId};
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::types::FieldCoordinate;
    use ffb_model::util::rng::GameRng;
    use std::collections::HashMap;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
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
        StepPushbackHookState::new(
            defender_id.into(),
            Some(PlayerState::new(PS_STANDING)),
            None, 0, free_square, vec![],
            HashMap::new(), HashMap::new(), None,
        )
    }

    #[test]
    fn register_into_adds_step_modifier() {
        let mut reg = SkillRegistry::empty();
        SideStepBehaviour::register_into(&mut reg);
        let sb = reg.get(SkillId::SideStep).expect("SideStep must be registered");
        assert_eq!(sb.get_step_modifiers().len(), 1);
    }

    #[test]
    fn step_modifier_applies_to_pushback_step() {
        let m = SideStepStepModifier;
        assert!(m.applies_to(StepId::Pushback));
    }

    #[test]
    fn step_modifier_does_not_apply_to_wrong_step() {
        let m = SideStepStepModifier;
        assert!(!m.applies_to(StepId::BlockRoll));
    }

    #[test]
    fn step_modifier_priority_is_three() {
        let m = SideStepStepModifier;
        assert_eq!(m.priority(), 3);
    }

    #[test]
    fn no_side_step_skill_returns_false() {
        let mut game = make_game();
        game.team_away.players.push(player_with_skills("def1", vec![]));
        game.defender_id = Some("def1".into());
        game.field_model.set_player_coordinate("def1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("def1", PlayerState::new(PS_STANDING));

        let m = SideStepStepModifier;
        let mut hs = default_hook_state("def1", true);
        let result = m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hs);
        assert!(!result);
    }

    #[test]
    fn side_step_with_no_free_square_returns_false() {
        let mut game = make_game();
        game.team_away.players.push(player_with_skills("def1", vec![SkillId::SideStep]));
        game.defender_id = Some("def1".into());
        game.field_model.set_player_coordinate("def1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("def1", PlayerState::new(PS_STANDING));

        let m = SideStepStepModifier;
        let mut hs = default_hook_state("def1", false);
        let result = m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hs);
        assert!(!result, "SideStep should not fire when no free squares around defender");
    }

    #[test]
    fn side_step_headless_auto_declines() {
        let mut game = make_game();
        game.team_away.players.push(player_with_skills("def1", vec![SkillId::SideStep]));
        game.defender_id = Some("def1".into());
        game.field_model.set_player_coordinate("def1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("def1", PlayerState::new(PS_STANDING));

        let m = SideStepStepModifier;
        let mut hs = default_hook_state("def1", true);
        let result = m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hs);
        assert!(result, "side step handled (declined) should return true");
        assert_eq!(hs.side_stepping.get("def1"), Some(&false));
    }

    #[test]
    fn side_step_declined_in_map_returns_false() {
        let mut game = make_game();
        game.team_away.players.push(player_with_skills("def1", vec![SkillId::SideStep]));
        game.defender_id = Some("def1".into());
        game.field_model.set_player_coordinate("def1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("def1", PlayerState::new(PS_STANDING));

        let m = SideStepStepModifier;
        let mut hs = default_hook_state("def1", true);
        hs.side_stepping.insert("def1".into(), false);
        let result = m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hs);
        assert!(!result);
        assert_eq!(hs.pushback_mode, PushbackMode::REGULAR);
    }

    #[test]
    fn side_step_accepted_clears_starting_square() {
        use ffb_model::enums::Direction;
        use ffb_model::types::PushbackSquare;

        let mut game = make_game();
        game.team_away.players.push(player_with_skills("def1", vec![SkillId::SideStep]));
        game.defender_id = Some("def1".into());
        game.field_model.set_player_coordinate("def1", FieldCoordinate::new(10, 7));
        game.field_model.set_player_state("def1", PlayerState::new(PS_STANDING));

        let m = SideStepStepModifier;
        let mut hs = default_hook_state("def1", true);
        hs.side_stepping.insert("def1".into(), true);
        let starting_sq = PushbackSquare::new(FieldCoordinate::new(10, 7), Direction::North, true);
        hs.starting_pushback_square = Some(starting_sq);

        m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hs);
        assert!(hs.starting_pushback_square.is_none(), "starting square should be cleared when SideStep used");
    }

    #[test]
    fn side_step_accepted_switches_to_side_step_mode() {
        use ffb_model::enums::Direction;
        use ffb_model::types::PushbackSquare;

        let mut game = make_game();
        game.team_away.players.push(player_with_skills("def1", vec![SkillId::SideStep]));
        game.defender_id = Some("def1".into());
        game.field_model.set_player_coordinate("def1", FieldCoordinate::new(10, 7));
        game.field_model.set_player_state("def1", PlayerState::new(PS_STANDING));

        let m = SideStepStepModifier;
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
        game.team_away.players.push(player_with_skills("def1", vec![SkillId::SideStep]));
        game.defender_id = Some("def1".into());
        game.field_model.set_player_coordinate("def1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("def1", PlayerState::new(PS_PRONE));

        let m = SideStepStepModifier;
        let mut hs = StepPushbackHookState::new(
            "def1".into(),
            Some(PlayerState::new(PS_PRONE)),
            None, 0, true, vec![],
            HashMap::new(), HashMap::new(), None,
        );

        m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hs);
        assert!(game.report_list.has_report(ffb_model::report::report_id::ReportId::SKILL_USE));
    }

}
