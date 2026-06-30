use ffb_model::enums::{PS_PRONE, PlayerState};
use ffb_model::enums::ReRollSource;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::dice_interpreter::DiceInterpreter;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::abstract_step_with_re_roll::ReRollState;
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};

/// Minimum MA to stand up for free. Java: Constant.MINIMUM_MOVE_TO_STAND_UP = 3.
const MINIMUM_MOVE_TO_STAND_UP: i32 = 3;

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.move.StepStandUp.
///
/// Stand-up sequence for a prone player.
///
/// If MA ≥ MINIMUM_MOVE_TO_STAND_UP (3) or player has `canStandUpForFree`: free stand-up, NEXT_STEP.
/// Otherwise: roll d6 ≥ 4 (with optional +modifier from allowStandUpAssists).
///   Success → NEXT_STEP; Failure → publish END_PLAYER_ACTION + GOTO failure label.
///
/// The outer guard is `actingPlayer.isStandingUp() && !actingPlayer.hasMoved()`, or
/// `reRolledAction == STAND_UP`.  If not standing up → NEXT_STEP immediately.
///
/// Re-roll: TRR offered via ReRollOffer prompt on first failure (no skill re-roll for StandUp in BB2025).
///
/// Init params: GOTO_LABEL_ON_FAILURE (mandatory).
///
/// setTurnStarted(true) and setConcessionPossible(false) are wired; isPinned() guard is wired.
/// DiceInterpreter::is_stand_up_successful is wired; modifier = 0 until stand-up assists are ported.
///
/// TODO: allowStandUpAssists → UtilPlayer.findStandUpAssists(game, player) not yet ported.
/// TODO: handleFailedStandUp (per-action turn data flags) not yet ported.
/// TODO: commitTargetSelection not yet ported.
pub struct StepStandUp {
    /// Java: fGotoLabelOnFailure
    pub goto_label_on_failure: String,
    /// Java: AbstractStepWithReRoll fields
    pub re_roll_state: ReRollState,
    /// Persisted roll for re-roll path
    roll: i32,
}

impl StepStandUp {
    pub fn new(goto_label_on_failure: String) -> Self {
        Self { goto_label_on_failure, re_roll_state: ReRollState::new(), roll: 0 }
    }
}

impl Step for StepStandUp {
    fn id(&self) -> StepId { StepId::StandUp }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if let Action::UseReRoll { use_reroll: false } = action {
            self.re_roll_state.re_roll_source = None;
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnFailure(v) => { self.goto_label_on_failure = v.clone(); true }
            _ => false,
        }
    }
}

impl StepStandUp {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: game.getTurnData().setTurnStarted(true)
        game.turn_data_mut().turn_started = true;

        let already_rerolled = self.re_roll_state.re_rolled_action
            .as_ref().map(|a| a.name == "STAND_UP").unwrap_or(false);

        // Java outer guard: (actingPlayer.isStandingUp() && !actingPlayer.hasMoved()) || STAND_UP == reRolledAction
        if !already_rerolled {
            if !game.acting_player.standing_up {
                return StepOutcome::next();
            }
            if game.acting_player.has_moved {
                return StepOutcome::next();
            }
        }

        // Java: game.setConcessionPossible(false)
        game.concession_possible = false;

        // Java: rollStandUp = player.getMovementWithModifiers() < 3 && !canStandUpForFree
        let roll_stand_up_needed = game.acting_player.player_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| {
                p.movement_with_modifiers() < MINIMUM_MOVE_TO_STAND_UP
                    && !p.has_skill_property(NamedProperties::CAN_STAND_UP_FOR_FREE)
            })
            .unwrap_or(true);

        if !roll_stand_up_needed {
            // MA >= 3 or has canStandUpForFree — stand up for free
            game.acting_player.has_moved = true;
            game.acting_player.standing_up = false;
            return StepOutcome::next();
        }

        // Java: if (STAND_UP == reRolledAction) { if (source == null || !useReRoll) → fail }
        if already_rerolled {
            let pid = game.acting_player.player_id.as_deref().unwrap_or("").to_owned();
            let source_opt = self.re_roll_state.re_roll_source.clone();
            let consumed = source_opt
                .as_ref()
                .map(|s| use_reroll(game, s, &pid))
                .unwrap_or(false);
            if !consumed {
                return self.fail_stand_up(game);
            }
            // Roll was reset to 0 when the re-roll offer was issued; a fresh d6 is rolled below
        }

        // Java: int roll = rollSkill()
        if self.roll == 0 {
            self.roll = rng.d6();
        }

        // Java: DiceInterpreter.isStandUpSuccessful(roll, modifier)
        // TODO: modifier from allowStandUpAssists
        let modifier = 0;
        let successful = DiceInterpreter::is_stand_up_successful(self.roll, modifier);

        // Java: if playerState.isPinned() → GOTO failure label (even on success)
        let is_pinned = game.acting_player.player_id.as_deref()
            .and_then(|id| game.field_model.player_state(id))
            .map(|s| s.is_pinned())
            .unwrap_or(false);
        if is_pinned {
            let label = self.goto_label_on_failure.clone();
            return StepOutcome::goto(&label)
                .publish(StepParameter::EndPlayerAction(true));
        }

        if successful {
            game.acting_player.has_moved = true;
            game.acting_player.standing_up = false;
            StepOutcome::next()
        } else {
            // Java: if (reRolledAction == STAND_UP || !askForReRollIfAvailable(...)) → handleFailedStandUp
            if already_rerolled {
                return self.fail_stand_up(game);
            }
            let minimum_roll = i32::max(2, 4 - modifier);
            if let Some(prompt) = ask_for_reroll_if_available(game, "STAND_UP", minimum_roll, false) {
                use ffb_model::model::re_rolled_action::ReRolledAction;
                self.re_roll_state.re_rolled_action = Some(ReRolledAction::new("STAND_UP"));
                self.re_roll_state.re_roll_source = Some(ReRollSource::new("TRR"));
                self.roll = 0; // reset so the re-roll gets a fresh d6
                return StepOutcome::cont().with_prompt(prompt);
            }
            self.fail_stand_up(game)
        }
    }

    fn fail_stand_up(&self, game: &mut Game) -> StepOutcome {
        // Java: setPlayerState(PRONE, !active), publish END_PLAYER_ACTION, GOTO failure label
        if let Some(pid) = game.acting_player.player_id.clone() {
            game.field_model.set_player_state(&pid, PlayerState::new(PS_PRONE));
        }
        // TODO: handleFailedStandUp (per-action turn data flags)
        let label = self.goto_label_on_failure.clone();
        StepOutcome::goto(&label)
            .publish(StepParameter::EndPlayerAction(true))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::{Rules, TurnMode};
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn not_standing_up_returns_next_step_immediately() {
        let mut game = make_game();
        game.acting_player.standing_up = false;
        let mut step = StepStandUp::new("fail".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.is_empty());
    }

    #[test]
    fn already_moved_returns_next_step_immediately() {
        let mut game = make_game();
        game.acting_player.standing_up = true;
        game.acting_player.has_moved = true;
        let mut step = StepStandUp::new("fail".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn success_clears_standing_up_flag() {
        for seed in 0u64..200 {
            let mut g = make_game();
            g.acting_player.standing_up = true;
            let mut step = StepStandUp::new("fail".into());
            let out = step.start(&mut g, &mut GameRng::new(seed));
            if out.action == StepAction::NextStep {
                assert!(!g.acting_player.standing_up);
                return;
            }
        }
    }

    #[test]
    fn failure_goes_to_failure_label_with_end_player_action() {
        let mut game = make_game();
        game.home_playing = true;
        game.turn_data_home.rerolls = 0;
        game.acting_player.standing_up = true;
        let mut step = StepStandUp::new("fail".into());
        step.roll = 1; // guaranteed fail (need >= 4)
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndPlayerAction(true))));
    }

    #[test]
    fn failure_with_trr_offers_reroll_prompt() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        game.acting_player.standing_up = true;
        let mut step = StepStandUp::new("fail".into());
        step.roll = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
        assert!(out.prompt.is_some());
    }

    #[test]
    fn accept_reroll_then_success_returns_next_step() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        game.acting_player.standing_up = true;
        let mut step = StepStandUp::new("fail".into());
        step.roll = 1;
        let _offer = step.start(&mut game, &mut GameRng::new(0));
        step.roll = 6; // success on re-roll
        let out = step.handle_command(&Action::UseReRoll { use_reroll: true }, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn decline_reroll_goes_to_failure_label() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        game.home_playing = true;
        game.turn_data_home.rerolls = 1;
        game.acting_player.standing_up = true;
        let mut step = StepStandUp::new("fail".into());
        step.roll = 1;
        let _offer = step.start(&mut game, &mut GameRng::new(0));
        let out = step.handle_command(&Action::UseReRoll { use_reroll: false }, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
    }

    #[test]
    fn set_parameter_goto_label_on_failure_accepted() {
        let mut step = StepStandUp::new("old".into());
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("new".into())));
        assert_eq!(step.goto_label_on_failure, "new");
    }
}
