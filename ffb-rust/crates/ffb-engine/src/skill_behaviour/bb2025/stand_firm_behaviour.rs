/// 1:1 translation of com.fumbbl.ffb.server.skillbehaviour.bb2025.StandFirmBehaviour.
///
/// Priority 2 modifier on StepPushback.
use crate::skill_behaviour::SkillBehaviour;
use crate::model::skill_behaviour::SkillBehaviour as SbContainer;
use crate::model::step_modifier::StepModifierTrait;
use crate::step::framework::StepId;
use crate::skill_behaviour::registry::SkillRegistry;
use crate::step::bb2025::block::step_pushback::StepPushbackHookState;
use ffb_model::enums::SkillId;
use ffb_model::model::game::Game;
use ffb_model::model::skill_use::SkillUse;
use ffb_model::report::report_skill_use::ReportSkillUse;

// ── StandFirmStepModifier ─────────────────────────────────────────────────────

pub struct StandFirmStepModifier;

impl StepModifierTrait for StandFirmStepModifier {
    fn applies_to(&self, step_id: StepId) -> bool { step_id == StepId::Pushback }

    fn priority(&self) -> i32 { 2 }

    /// Java: StandFirmBehaviour.handleExecuteStepHook(StepPushback step, StepState state)
    ///
    /// 1. Auto-stand-firm if Rooted.
    /// 2. Auto NOT stand-firm if no tacklezones (out of play).
    /// 3. Auto NOT stand-firm if Blitz + cancelling skill (Juggernaut) from different team.
    /// 4. If defender has StandFirm and standing_firm map defaults to true:
    ///    - If decision not yet made: headless → auto-decline (false).
    ///    - If standing firm: cancel the push (do_push=true, clear stack).
    /// Returns true if standing firm resolved the push; false otherwise.
    fn handle_execute_step(
        &self,
        game: &mut Game,
        step_state: &mut dyn std::any::Any,
    ) -> bool {
        let state = step_state
            .downcast_mut::<StepPushbackHookState>()
            .expect("StandFirmStepModifier: step_state must be StepPushbackHookState");

        let defender_id = state.defender_id.clone();

        // Java: UtilCards.hasSkill(state.defender, skill)
        let has_stand_firm = game.player(&defender_id)
            .map(|p| p.has_skill(SkillId::StandFirm))
            .unwrap_or(false);

        // Java: UtilCards.getSkillCancelling(actingPlayer.getPlayer(), skill)
        // Cancelling skill = one that has cancelsCanRefuseToBePushed property
        let cancelling_skill = game.acting_player.player_id.as_deref()
            .and_then(|id| game.player(id))
            .and_then(|p| {
                p.all_skill_ids().find(|id| {
                    id.properties().contains(&"cancelsCanRefuseToBePushed")
                })
            });

        // Java: playerState = game.getFieldModel().getPlayerState(state.defender)
        let defender_state = game.field_model.player_state(&defender_id);

        // Java: if (playerState.isRooted()) state.standingFirm.put(id, true)
        if defender_state.map(|s| s.is_rooted()).unwrap_or(false) {
            state.standing_firm.insert(defender_id.clone(), true);
        } else {
            // Java: else if (no tacklezones) addReport(NO_TACKLEZONE); state.standingFirm.put(id, false)
            let has_tacklezones = defender_state.map(|s| s.has_tacklezones()).unwrap_or(true);
            let old_has_tacklezones = state.old_defender_state.map(|s| s.has_tacklezones()).unwrap_or(true);
            if has_stand_firm && (
                (state.pushback_stack_len == 0 && !old_has_tacklezones)
                || (state.pushback_stack_len > 0 && !has_tacklezones)
            ) {
                state.standing_firm.insert(defender_id.clone(), false);
                game.report_list.add(ReportSkillUse::new(
                    Some(defender_id.clone()),
                    SkillId::StandFirm,
                    false,
                    SkillUse::NO_TACKLEZONE,
                ));
            } else if let Some(cancel_skill) = cancelling_skill {
                // Java: (PlayerAction.BLITZ == actingPlayer.getPlayerAction()) && cancellingSkill != null
                //       && hasSkill(defender, skill) && defender.getTeam() != actingPlayer.getTeam()
                let is_blitz = game.acting_player.player_action
                    .map(|a| a.is_blitzing())
                    .unwrap_or(false);
                if has_stand_firm && is_blitz {
                    let attacker_id = game.acting_player.player_id.clone().unwrap_or_default();
                    let attacker_home = game.team_home.player(&attacker_id).is_some();
                    let defender_home = game.team_home.player(&defender_id).is_some();
                    if attacker_home != defender_home {
                        state.standing_firm.insert(defender_id.clone(), false);
                        // Java: addReport(ReportSkillUse(actingPlayerId, cancellingSkill, true, CANCEL_STAND_FIRM))
                        game.report_list.add(ReportSkillUse::new(
                            Some(attacker_id),
                            cancel_skill,
                            true,
                            SkillUse::CANCEL_STAND_FIRM,
                        ));
                    }
                }
            }
        }

        // Java: if (UtilCards.hasSkill(state.defender, skill)
        //          && state.standingFirm.getOrDefault(state.defender.getId(), true))
        if !has_stand_firm {
            return false;
        }

        let using_stand_firm = *state.standing_firm.get(&defender_id).unwrap_or(&true);
        if !using_stand_firm {
            return false;
        }

        // Java: if (!state.standingFirm.containsKey(id)) show dialog → headless: auto-decline
        if !state.standing_firm.contains_key(&defender_id) {
            // Headless: auto-decline (false = do not use stand firm)
            state.standing_firm.insert(defender_id.clone(), false);
            return false;
        }

        // Java: state.doPush = true; pushbackStack.clear(); publish StartingPushbackSquare(null); etc.
        state.do_push = true;
        state.pushback_squares.clear();
        state.starting_pushback_square = None;

        // Java: addReport(ReportSkillUse(id, skill, true, AVOID_PUSH))
        game.report_list.add(ReportSkillUse::new(
            Some(defender_id.clone()),
            SkillId::StandFirm,
            true,
            SkillUse::AVOID_PUSH,
        ));

        true
    }
}

// ── StandFirmBehaviour ────────────────────────────────────────────────────────

/// Stand Firm: player may ignore a push result and stay in their square.
pub struct StandFirmBehaviour;

impl StandFirmBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for StandFirmBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for StandFirmBehaviour {
    fn name(&self) -> &'static str { "StandFirmBehaviour" }

    fn execute_step_hook(&self, game: &mut ffb_model::model::game::Game) -> bool {
        // Legacy hook path — logic lives in StandFirmStepModifier.
        let has_skill = game.acting_player.player_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| p.has_skill(SkillId::StandFirm))
            .unwrap_or(false);
        if !has_skill { return false; }
        false
    }
}

impl StandFirmBehaviour {
    pub fn register_into(registry: &mut SkillRegistry) {
        let mut sb = SbContainer::new();
        sb.register_step_modifier(Box::new(StandFirmStepModifier));
        registry.register(SkillId::StandFirm, sb);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skill_behaviour::registry::SkillRegistry;
    use crate::step::bb2025::block::step_pushback::StepPushbackHookState;
    use crate::step::framework::test_team;
    use ffb_model::enums::{PlayerState, PS_STANDING, Rules, SkillId};
    use ffb_model::model::game::Game;
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::types::FieldCoordinate;
    use ffb_model::enums::PS_PRONE;
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

    fn default_hook_state(defender_id: &str) -> StepPushbackHookState {
        StepPushbackHookState::new(
            defender_id.into(), None, None, 0, true, vec![],
            HashMap::new(), HashMap::new(), None,
        )
    }

    #[test]
    fn register_into_adds_step_modifier() {
        let mut reg = SkillRegistry::empty();
        StandFirmBehaviour::register_into(&mut reg);
        let sb = reg.get(SkillId::StandFirm).expect("StandFirm must be registered");
        assert_eq!(sb.get_step_modifiers().len(), 1);
    }

    #[test]
    fn step_modifier_applies_to_pushback_step() {
        let m = StandFirmStepModifier;
        assert!(m.applies_to(StepId::Pushback));
    }

    #[test]
    fn step_modifier_does_not_apply_to_wrong_step() {
        let m = StandFirmStepModifier;
        assert!(!m.applies_to(StepId::BlockRoll));
    }

    #[test]
    fn step_modifier_priority_is_two() {
        let m = StandFirmStepModifier;
        assert_eq!(m.priority(), 2);
    }

    #[test]
    fn no_stand_firm_skill_returns_false() {
        let mut game = make_game();
        game.team_away.players.push(player_with_skills("def1", vec![]));
        game.field_model.set_player_coordinate("def1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("def1", PlayerState::new(PS_STANDING));

        let m = StandFirmStepModifier;
        let mut hs = default_hook_state("def1");
        let result = m.handle_execute_step(&mut game, &mut hs);
        assert!(!result);
    }

    #[test]
    fn stand_firm_not_decided_headless_auto_declines() {
        // Defender has StandFirm, no pre-populated standing_firm map → auto-decline
        let mut game = make_game();
        game.team_away.players.push(player_with_skills("def1", vec![SkillId::StandFirm]));
        game.field_model.set_player_coordinate("def1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("def1", PlayerState::new(PS_STANDING));

        let m = StandFirmStepModifier;
        let mut hs = default_hook_state("def1");
        // standing_firm map is empty → auto-decline
        let result = m.handle_execute_step(&mut game, &mut hs);
        // Should return false (declined) and have inserted false
        assert!(!result);
        assert_eq!(hs.standing_firm.get("def1"), Some(&false));
    }

    #[test]
    fn stand_firm_accepted_cancels_push() {
        // Defender has StandFirm, standing_firm pre-set to true → stand firm used
        let mut game = make_game();
        game.team_away.players.push(player_with_skills("def1", vec![SkillId::StandFirm]));
        game.field_model.set_player_coordinate("def1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("def1", PlayerState::new(PS_STANDING));

        let m = StandFirmStepModifier;
        let mut hs = default_hook_state("def1");
        hs.standing_firm.insert("def1".into(), true);

        let result = m.handle_execute_step(&mut game, &mut hs);
        assert!(result, "should return true when stand firm is accepted");
        assert!(hs.do_push, "do_push should be true after stand firm");
        assert!(hs.starting_pushback_square.is_none(), "starting square should be cleared");
        assert!(hs.pushback_squares.is_empty(), "pushback squares should be cleared");
    }

    #[test]
    fn stand_firm_accepted_adds_avoid_push_report() {
        let mut game = make_game();
        game.team_away.players.push(player_with_skills("def1", vec![SkillId::StandFirm]));
        game.field_model.set_player_coordinate("def1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("def1", PlayerState::new(PS_STANDING));

        let m = StandFirmStepModifier;
        let mut hs = default_hook_state("def1");
        hs.standing_firm.insert("def1".into(), true);
        m.handle_execute_step(&mut game, &mut hs);

        assert!(game.report_list.has_report(ffb_model::report::report_id::ReportId::SKILL_USE));
    }

    #[test]
    fn stand_firm_declined_returns_false() {
        let mut game = make_game();
        game.team_away.players.push(player_with_skills("def1", vec![SkillId::StandFirm]));
        game.field_model.set_player_coordinate("def1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("def1", PlayerState::new(PS_STANDING));

        let m = StandFirmStepModifier;
        let mut hs = default_hook_state("def1");
        hs.standing_firm.insert("def1".into(), false);

        let result = m.handle_execute_step(&mut game, &mut hs);
        assert!(!result, "should return false when stand firm declined");
        assert!(!hs.do_push, "do_push should stay false when declined");
    }

    #[test]
    fn name_is_not_empty() {
        assert!(!StandFirmBehaviour::new().name().is_empty());
    }

    #[test]
    fn execute_step_hook_returns_false() {
        let b = StandFirmBehaviour::new();
        let mut game = Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2025,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = StandFirmBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }

    #[test]
    fn stand_firm_with_no_tacklezone_auto_declines() {
        let mut game = make_game();
        game.team_away.players.push(player_with_skills("def1", vec![SkillId::StandFirm]));
        game.field_model.set_player_coordinate("def1", FieldCoordinate::new(5, 5));
        // Prone state has no tacklezones
        game.field_model.set_player_state("def1", PlayerState::new(PS_PRONE));

        let m = StandFirmStepModifier;
        let mut hs = default_hook_state("def1");
        // old_defender_state also has no tacklezones (prone) → auto-decline
        hs.old_defender_state = Some(PlayerState::new(PS_PRONE));
        m.handle_execute_step(&mut game, &mut hs);
        // Should NOT stand firm (inserted false)
        assert_eq!(hs.standing_firm.get("def1"), Some(&false));
    }
}
