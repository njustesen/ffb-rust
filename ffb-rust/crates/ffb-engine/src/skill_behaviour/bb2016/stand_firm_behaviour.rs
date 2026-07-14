/// 1:1 translation of com.fumbbl.ffb.server.skillbehaviour.bb2016.StandFirmBehaviour.
///
/// Priority 1 modifier on StepPushback.
///
/// **BB2016 vs BB2020/BB2025 differences:** the auto-decline branch checks
/// `isProneOrStunned()`/`isStunned()` (current and old state) instead of the hasTacklezones-based
/// chain used by later editions, and its Juggernaut-cancel branch checks direct coordinate
/// adjacency (same as BB2020's, unlike BB2025's simplified team-check).
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

    fn priority(&self) -> i32 { 1 }

    /// Java: StandFirmBehaviour.handleExecuteStepHook(StepPushback step, StepState state)
    fn handle_execute_step(
        &self,
        game: &mut Game,
        _rng: &mut ffb_model::util::rng::GameRng,
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
        let cancelling_skill = game.acting_player.player_id.as_deref()
            .and_then(|id| game.player(id))
            .and_then(|p| {
                p.all_skill_ids().find(|id| {
                    id.properties().contains(&"cancelsCanRefuseToBePushed")
                })
            });

        let defender_state = game.field_model.player_state(&defender_id);

        // Java: if (playerState.isRooted()) standingFirm.put(id, true)
        if defender_state.map(|s| s.is_rooted()).unwrap_or(false) {
            state.standing_firm.insert(defender_id.clone(), true);
        } else {
            // Java: else if (isProneOrStunned || isStunned || (oldDefenderState != null &&
            //         (oldDefenderState.isProneOrStunned() || oldDefenderState.isStunned())))
            let is_prone_or_stunned = defender_state
                .map(|s| s.is_prone_or_stunned() || s.is_stunned())
                .unwrap_or(false);
            let old_is_prone_or_stunned = state.old_defender_state
                .map(|s| s.is_prone_or_stunned() || s.is_stunned())
                .unwrap_or(false);
            if is_prone_or_stunned || old_is_prone_or_stunned {
                state.standing_firm.insert(defender_id.clone(), false);
            } else if let Some(cancel_skill) = cancelling_skill {
                // Java: BLITZ == actingPlayer.getPlayerAction() && cancellingSkill != null
                //       && hasSkill(defender, skill)
                //       && getPlayerCoordinate(actingPlayer).isAdjacent(getPlayerCoordinate(defender))
                let is_blitz = game.acting_player.player_action
                    .map(|a| a.is_blitzing())
                    .unwrap_or(false);
                let attacker_id = game.acting_player.player_id.clone().unwrap_or_default();
                let is_adjacent = game.field_model.player_coordinate(&attacker_id)
                    .zip(game.field_model.player_coordinate(&defender_id))
                    .map(|(a, d)| a.is_adjacent(d))
                    .unwrap_or(false);
                if has_stand_firm && is_blitz && is_adjacent {
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

        // Java: if (hasSkill(defender, skill) && standingFirm.getOrDefault(id, true))
        if !has_stand_firm {
            return false;
        }

        let using_stand_firm = *state.standing_firm.get(&defender_id).unwrap_or(&true);
        if !using_stand_firm {
            return false;
        }

        // Java: if (!standingFirm.containsKey(id)) show dialog → headless: auto-decline
        // (established precedent: see StandFirmStepModifier in bb2025, Phase AAF Dodge convention)
        if !state.standing_firm.contains_key(&defender_id) {
            state.standing_firm.insert(defender_id.clone(), false);
            return false;
        }

        // Java: state.doPush = true; pushbackStack.clear(); publish STARTING_PUSHBACK_SQUARE=null;
        //       publish FOLLOWUP_CHOICE=false; addReport(AVOID_PUSH)
        state.do_push = true;
        state.pushback_squares.clear();
        state.starting_pushback_square = None;

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

    pub fn register_into(registry: &mut SkillRegistry) {
        let mut sb = SbContainer::new();
        sb.register_step_modifier(Box::new(StandFirmStepModifier));
        registry.register(SkillId::StandFirm, sb);
    }
}

impl Default for StandFirmBehaviour {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skill_behaviour::registry::SkillRegistry;
    use crate::step::bb2025::block::step_pushback::StepPushbackHookState;
    use crate::step::framework::test_team;
    use ffb_model::enums::{PlayerState, PS_STANDING, PS_PRONE, Rules, SkillId};
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::types::FieldCoordinate;
    use ffb_model::util::rng::GameRng;
    use std::collections::HashMap;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
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
    fn step_modifier_priority_is_one() {
        let m = StandFirmStepModifier;
        assert_eq!(m.priority(), 1);
    }

    #[test]
    fn no_stand_firm_skill_returns_false() {
        let mut game = make_game();
        game.team_away.players.push(player_with_skills("def1", vec![]));
        game.field_model.set_player_coordinate("def1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("def1", PlayerState::new(PS_STANDING));

        let m = StandFirmStepModifier;
        let mut hs = default_hook_state("def1");
        let result = m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hs);
        assert!(!result);
    }

    #[test]
    fn stand_firm_not_decided_headless_auto_declines() {
        let mut game = make_game();
        game.team_away.players.push(player_with_skills("def1", vec![SkillId::StandFirm]));
        game.field_model.set_player_coordinate("def1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("def1", PlayerState::new(PS_STANDING));

        let m = StandFirmStepModifier;
        let mut hs = default_hook_state("def1");
        let result = m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hs);
        assert!(!result);
        assert_eq!(hs.standing_firm.get("def1"), Some(&false));
    }

    #[test]
    fn stand_firm_accepted_cancels_push() {
        let mut game = make_game();
        game.team_away.players.push(player_with_skills("def1", vec![SkillId::StandFirm]));
        game.field_model.set_player_coordinate("def1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("def1", PlayerState::new(PS_STANDING));

        let m = StandFirmStepModifier;
        let mut hs = default_hook_state("def1");
        hs.standing_firm.insert("def1".into(), true);

        let result = m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hs);
        assert!(result, "should return true when stand firm is accepted");
        assert!(hs.do_push, "do_push should be true after stand firm");
        assert!(hs.starting_pushback_square.is_none(), "starting square should be cleared");
        assert!(hs.pushback_squares.is_empty(), "pushback squares should be cleared");
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

        let result = m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hs);
        assert!(!result, "should return false when stand firm declined");
        assert!(!hs.do_push, "do_push should stay false when declined");
    }

    /// BB2016-specific: prone/stunned defender auto-declines stand firm.
    #[test]
    fn stand_firm_prone_defender_auto_declines() {
        let mut game = make_game();
        game.team_away.players.push(player_with_skills("def1", vec![SkillId::StandFirm]));
        game.field_model.set_player_coordinate("def1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("def1", PlayerState::new(PS_PRONE));

        let m = StandFirmStepModifier;
        let mut hs = default_hook_state("def1");
        m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hs);
        assert_eq!(hs.standing_firm.get("def1"), Some(&false));
    }

    #[test]
    fn stand_firm_rooted_auto_stands_firm() {
        let mut game = make_game();
        game.team_away.players.push(player_with_skills("def1", vec![SkillId::StandFirm]));
        game.field_model.set_player_coordinate("def1", FieldCoordinate::new(5, 5));
        let rooted = PlayerState::new(PS_STANDING).change_rooted(true);
        game.field_model.set_player_state("def1", rooted);

        let m = StandFirmStepModifier;
        let mut hs = default_hook_state("def1");
        m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hs);
        assert_eq!(hs.standing_firm.get("def1"), Some(&true));
    }

}
