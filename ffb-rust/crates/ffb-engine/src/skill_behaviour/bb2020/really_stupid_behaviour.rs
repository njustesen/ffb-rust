/// 1:1 translation of com.fumbbl.ffb.server.skillbehaviour.bb2020.ReallyStupidBehaviour.
///
/// **BB2020 vs BB2025 difference (cancelPlayerAction):**
/// - THROW_TEAM_MATE / THROW_TEAM_MATE_MOVE → `setPassUsed(true)` (BB2025 uses `setTtmUsed`).
/// - No SECURE_THE_BALL or PUNT cases (BB2025 additions).
/// - `commitTargetSelection()` is called before the roll (same as BB2025; BB2016 omits it).
/// - `targetSelectionState.failed()` is called on failure (same as BB2025; BB2016 omits it).
use crate::model::skill_behaviour::SkillBehaviour as SbContainer;
use crate::model::step_modifier::StepModifierTrait;
use crate::step::framework::StepId;
use crate::step::action::common::step_really_stupid::{StepReallyStupidHookState, has_non_really_stupid_adjacent_teammate};
use crate::skill_behaviour::registry::SkillRegistry;
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};
use ffb_model::enums::{PlayerAction, PS_PRONE, ReRollSource, SkillId};
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_mechanics::mechanics::minimum_roll_confusion;
use ffb_model::report::report_confusion_roll::ReportConfusionRoll;
use crate::step::framework::{StepOutcome, StepParameter};

pub struct ReallyStupidBehaviour;

impl ReallyStupidBehaviour {
    pub fn new() -> Self { Self }

    pub fn register_into(registry: &mut SkillRegistry) {
        let mut sb = SbContainer::new();
        sb.register_step_modifier(Box::new(ReallyStupidStepModifier));
        registry.register(SkillId::ReallyStupid, sb);
    }

    /// Classify the player action into the BB2020 turn-data flag that must be marked used when
    /// the really-stupid roll causes the action to be cancelled.
    ///
    /// Returns one of the `TurnDataFlag` variants. Mirrors Java
    /// `cancelPlayerAction(StepReallyStupid)` in the BB2020 edition.
    pub fn turn_data_flag_for_action_bb2020(action: PlayerActionKind) -> TurnDataFlag {
        match action {
            // Blitz actions
            PlayerActionKind::Blitz
            | PlayerActionKind::BlitzMove
            | PlayerActionKind::KickEmBlitz => TurnDataFlag::BlitzUsed,

            // Kick-team-mate
            PlayerActionKind::KickTeamMate
            | PlayerActionKind::KickTeamMateMove => TurnDataFlag::KtmUsed,

            // BB2020: both pass AND throw-team-mate use the same passUsed flag.
            PlayerActionKind::Pass
            | PlayerActionKind::PassMove
            | PlayerActionKind::ThrowTeamMate
            | PlayerActionKind::ThrowTeamMateMove => TurnDataFlag::PassUsed,

            // Hand-over
            PlayerActionKind::HandOver
            | PlayerActionKind::HandOverMove => TurnDataFlag::HandOverUsed,

            // Foul (only if no extra-foul property)
            PlayerActionKind::Foul
            | PlayerActionKind::FoulMove => TurnDataFlag::FoulUsed,

            // BB2020 does NOT have SECURE_THE_BALL or PUNT — no flag change.
            _ => TurnDataFlag::None,
        }
    }
}

/// Thin classification of player action kinds relevant to turn-data cancellation.
/// Only the variants that differ between BB2020 and BB2025 are separated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerActionKind {
    Blitz,
    BlitzMove,
    KickEmBlitz,
    KickTeamMate,
    KickTeamMateMove,
    Pass,
    PassMove,
    ThrowTeamMate,
    ThrowTeamMateMove,
    HandOver,
    HandOverMove,
    Foul,
    FoulMove,
    Other,
}

/// Which turn-data boolean flag to mark as used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnDataFlag {
    BlitzUsed,
    KtmUsed,
    /// BB2020: a single flag covers both Pass and ThrowTeamMate.
    PassUsed,
    HandOverUsed,
    FoulUsed,
    None,
}

impl Default for ReallyStupidBehaviour {
    fn default() -> Self { Self::new() }
}

// ── BB2020 cancel helper ──────────────────────────────────────────────────────
//
// Java BB2020 ReallyStupidBehaviour.cancelPlayerAction (same structure as BB2020 BoneHead):
//   BLITZ/BLITZ_MOVE/KICK_EM_BLITZ → blitzUsed
//   KICK_TEAM_MATE/KICK_TEAM_MATE_MOVE → ktmUsed   (differs from BB2016: was blitzUsed)
//   PASS/PASS_MOVE/THROW_TEAM_MATE/THROW_TEAM_MATE_MOVE → passUsed  (differs from BB2025: TTM was ttmUsed)
//   HAND_OVER/HAND_OVER_MOVE → handOverUsed
//   FOUL/FOUL_MOVE (no allowsAdditionalFoul) → foulUsed
fn cancel_bb2020_really_stupid(game: &mut Game, player_id: &str) {
    match game.acting_player.player_action {
        Some(PlayerAction::Blitz)
        | Some(PlayerAction::BlitzMove)
        | Some(PlayerAction::KickEmBlitz) => {
            game.turn_data_mut().blitz_used = true;
        }
        Some(PlayerAction::KickTeamMate) | Some(PlayerAction::KickTeamMateMove) => {
            game.turn_data_mut().ktm_used = true;
        }
        // BB2020: ThrowTeamMate uses passUsed (not ttmUsed as in BB2025)
        Some(PlayerAction::Pass)
        | Some(PlayerAction::PassMove)
        | Some(PlayerAction::ThrowTeamMate)
        | Some(PlayerAction::ThrowTeamMateMove) => {
            game.turn_data_mut().pass_used = true;
        }
        Some(PlayerAction::HandOver) | Some(PlayerAction::HandOverMove) => {
            game.turn_data_mut().hand_over_used = true;
        }
        Some(PlayerAction::Foul) | Some(PlayerAction::FoulMove) => {
            game.turn_data_mut().foul_used = true;
        }
        _ => {}
    }

    if let Some(state) = game.field_model.player_state(player_id) {
        let new_state = if game.acting_player.standing_up {
            state.change_base(PS_PRONE).change_active(false)
        } else {
            state.change_confused(true).change_active(false)
        };
        game.field_model.set_player_state(player_id, new_state);
    }

    game.pass_coordinate = None;
}

// ── ReallyStupidStepModifier ──────────────────────────────────────────────────

pub struct ReallyStupidStepModifier;

impl StepModifierTrait for ReallyStupidStepModifier {
    fn applies_to(&self, step_id: StepId) -> bool { step_id == StepId::ReallyStupid }

    fn priority(&self) -> i32 { 0 }

    /// Java: bb2020.ReallyStupidBehaviour.handleExecuteStepHook(StepReallyStupid step, StepState state)
    fn handle_execute_step(
        &self,
        game: &mut Game,
        rng: &mut GameRng,
        step_state: &mut dyn std::any::Any,
    ) -> bool {
        let state = step_state
            .downcast_mut::<StepReallyStupidHookState>()
            .expect("ReallyStupidStepModifier: step_state must be StepReallyStupidHookState");

        // Java: if (!game.getTurnMode().checkNegatraits()) { setNextAction(NEXT_STEP); return false; }
        if !game.turn_mode.check_negatraits() {
            state.outcome = Some(StepOutcome::next());
            return false;
        }

        let player_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => { state.outcome = Some(StepOutcome::next()); return false; }
        };

        let has_really_stupid = game.player(&player_id)
            .map(|p| p.has_skill(SkillId::ReallyStupid))
            .unwrap_or(false);
        if !has_really_stupid {
            state.outcome = Some(StepOutcome::next());
            return false;
        }

        // Java: if (REALLY_STUPID == reRolledAction && !useReRoll) doRoll = false
        let mut skip_roll = false;
        if state.re_rolled_action.as_deref() == Some("REALLY_STUPID") {
            if let Some(ref source_name) = state.re_roll_source.clone() {
                let source = ReRollSource::new(source_name.as_str());
                if !use_reroll(game, &source, &player_id) {
                    skip_roll = true;
                }
            } else {
                skip_roll = true; // player declined
            }
        }

        // goodConditions: TTM/KTM actions get good conditions, or adjacent non-RS teammate present
        let good_conditions = matches!(game.acting_player.player_action,
            Some(PlayerAction::ThrowTeamMate) | Some(PlayerAction::ThrowTeamMateMove)
            | Some(PlayerAction::KickTeamMate) | Some(PlayerAction::KickTeamMateMove)
        ) || has_non_really_stupid_adjacent_teammate(game, &player_id);

        let min_roll = minimum_roll_confusion(good_conditions);

        if skip_roll {
            let confusion_event = GameEvent::ConfusionRoll { player_id: player_id.clone(), roll: 1, confused: true };
            cancel_bb2020_really_stupid(game, &player_id);
            state.outcome = Some(
                StepOutcome::goto(&state.goto_label_on_failure)
                    .with_event(confusion_event)
                    .publish(StepParameter::EndPlayerAction(true))
            );
            return false;
        }

        // BB2020: commitTargetSelection() before roll (same as BB2025, unlike BB2016)
        if let Some(ref mut ts) = game.field_model.target_selection_state {
            ts.commit();
        }
        let roll = rng.d6();
        let successful = roll >= min_roll;

        // Java: actingPlayer.markSkillUsed(skill)
        let is_home = game.team_home.player(&player_id).is_some();
        if is_home {
            if let Some(p) = game.team_home.player_mut(&player_id) {
                p.used_skills.insert(SkillId::ReallyStupid);
            }
        } else if let Some(p) = game.team_away.player_mut(&player_id) {
            p.used_skills.insert(SkillId::ReallyStupid);
        }

        let re_rolled = state.re_rolled_action.as_deref() == Some("REALLY_STUPID")
            && state.re_roll_source.is_some();
        game.report_list.add(ReportConfusionRoll::new(
            Some(player_id.clone()),
            successful,
            roll,
            min_roll,
            re_rolled,
            Some(SkillId::ReallyStupid.class_name().to_string()),
        ));
        let confusion_event = GameEvent::ConfusionRoll { player_id: player_id.clone(), roll, confused: !successful };

        if successful {
            state.outcome = Some(StepOutcome::next().with_event(confusion_event));
        } else if state.re_rolled_action.is_none() {
            if let Some(prompt) = ask_for_reroll_if_available(game, "REALLY_STUPID", min_roll, false) {
                state.updated_re_rolled_action = Some("REALLY_STUPID".into());
                state.updated_re_roll_source = Some("TRR".into());
                state.outcome = Some(StepOutcome::cont().with_event(confusion_event).with_prompt(prompt));
                return false;
            }
            cancel_bb2020_really_stupid(game, &player_id);
            state.outcome = Some(
                StepOutcome::goto(&state.goto_label_on_failure)
                    .with_event(confusion_event)
                    .publish(StepParameter::EndPlayerAction(true))
            );
        } else {
            cancel_bb2020_really_stupid(game, &player_id);
            state.outcome = Some(
                StepOutcome::goto(&state.goto_label_on_failure)
                    .with_event(confusion_event)
                    .publish(StepParameter::EndPlayerAction(true))
            );
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use crate::step::framework::test_team;

    fn test_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2020)
    }

    // --- BB2020 action-cancellation mapping tests ---

    /// BB2020: ThrowTeamMate maps to PassUsed (not TtmUsed).
    #[test]
    fn throw_team_mate_uses_pass_flag_in_bb2020() {
        assert_eq!(
            ReallyStupidBehaviour::turn_data_flag_for_action_bb2020(PlayerActionKind::ThrowTeamMate),
            TurnDataFlag::PassUsed,
            "BB2020 should mark passUsed for ThrowTeamMate"
        );
    }

    /// BB2020: ThrowTeamMateMove also maps to PassUsed.
    #[test]
    fn throw_team_mate_move_uses_pass_flag_in_bb2020() {
        assert_eq!(
            ReallyStupidBehaviour::turn_data_flag_for_action_bb2020(PlayerActionKind::ThrowTeamMateMove),
            TurnDataFlag::PassUsed
        );
    }

    /// BB2020: Pass maps to PassUsed (same as BB2025).
    #[test]
    fn pass_action_uses_pass_flag() {
        assert_eq!(
            ReallyStupidBehaviour::turn_data_flag_for_action_bb2020(PlayerActionKind::Pass),
            TurnDataFlag::PassUsed
        );
    }

    /// BB2020: Blitz maps to BlitzUsed.
    #[test]
    fn blitz_uses_blitz_flag() {
        assert_eq!(
            ReallyStupidBehaviour::turn_data_flag_for_action_bb2020(PlayerActionKind::Blitz),
            TurnDataFlag::BlitzUsed
        );
    }

    /// BB2020: unknown/other action has no flag effect.
    #[test]
    fn other_action_has_no_flag() {
        assert_eq!(
            ReallyStupidBehaviour::turn_data_flag_for_action_bb2020(PlayerActionKind::Other),
            TurnDataFlag::None
        );
    }

    // --- infrastructure tests ---

    #[test]
    fn register_into_adds_step_modifier() {
        let mut reg = SkillRegistry::empty();
        ReallyStupidBehaviour::register_into(&mut reg);
        let sb = reg.get(SkillId::ReallyStupid).expect("ReallyStupid must be registered");
        assert_eq!(sb.get_step_modifiers().len(), 1);
    }

    #[test]
    fn step_modifier_applies_to_correct_step() {
        let m = ReallyStupidStepModifier;
        assert!(m.applies_to(StepId::ReallyStupid));
    }

    #[test]
    fn step_modifier_does_not_apply_to_wrong_step() {
        let m = ReallyStupidStepModifier;
        assert!(!m.applies_to(StepId::BlockRoll));
    }

    #[test]
    fn step_modifier_no_negatraits_gives_next() {
        use ffb_model::enums::TurnMode;
        let m = ReallyStupidStepModifier;
        let mut game = test_game();
        game.turn_mode = TurnMode::KickoffReturn;
        let mut hook = StepReallyStupidHookState {
            goto_label_on_failure: "FAIL".into(),
            re_rolled_action: None,
            re_roll_source: None,
            outcome: None,
            updated_re_rolled_action: None,
            updated_re_roll_source: None,
        };
        m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hook);
        assert!(hook.outcome.is_some());
    }

    /// BB2020: ThrowTeamMate cancel maps to pass_used (not ttm_used as in BB2025)
    #[test]
    fn cancel_bb2020_ttm_sets_pass_used_not_ttm_used() {
        let mut game = test_game();
        game.acting_player.player_action = Some(PlayerAction::ThrowTeamMate);
        cancel_bb2020_really_stupid(&mut game, "nobody");
        assert!(game.turn_data_mut().pass_used);
        assert!(!game.turn_data_mut().ttm_used);
    }

    /// BB2020: KickTeamMate cancel maps to ktm_used (not blitz_used as in BB2016)
    #[test]
    fn cancel_bb2020_ktm_sets_ktm_used_not_blitz_used() {
        let mut game = test_game();
        game.acting_player.player_action = Some(PlayerAction::KickTeamMate);
        cancel_bb2020_really_stupid(&mut game, "nobody");
        assert!(game.turn_data_mut().ktm_used);
        assert!(!game.turn_data_mut().blitz_used);
    }
}
