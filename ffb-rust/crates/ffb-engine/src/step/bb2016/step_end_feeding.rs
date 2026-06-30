use ffb_model::enums::{InducementPhase, TurnMode};
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_steps::check_touchdown;
use crate::step::generator::mixed::EndTurn;
use crate::step::generator::bb2016::{Pass, Select};
use crate::step::generator::bb2016::pass::PassParams;
use crate::step::generator::bb2016::select::SelectParams;
use crate::step::generator::common::Inducement;
use crate::step::generator::common::inducement::InducementParams;

/// Final step of the feed sequence (BB2016). Consumes EndPlayerAction/EndTurn.
/// 1:1 translation of com.fumbbl.ffb.server.step.bb2016.StepEndFeeding.
pub struct StepEndFeeding {
    /// Java: fEndPlayerAction
    pub end_player_action: bool,
    /// Java: fEndTurn
    pub end_turn: bool,
}

impl StepEndFeeding {
    pub fn new() -> Self {
        Self { end_player_action: false, end_turn: false }
    }
}

impl Default for StepEndFeeding {
    fn default() -> Self { Self::new() }
}

impl Step for StepEndFeeding {
    fn id(&self) -> StepId { StepId::EndFeeding }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::EndPlayerAction(v) => { self.end_player_action = *v; true }
            StepParameter::EndTurn(v) => { self.end_turn = *v; true }
            _ => false,
        }
    }
}

impl StepEndFeeding {
    fn execute_step(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: UtilServerDialog.hideDialog(getGameState()) — no-op in headless Rust
        // Java: fEndTurn |= UtilServerSteps.checkTouchdown(getGameState())
        self.end_turn |= check_touchdown(game);

        if self.end_turn {
            if game.turn_mode == TurnMode::PassBlock {
                // Java: EndTurn.pushSequence(new EndTurn.SequenceParams(getGameState(), false))
                return StepOutcome::next().push_seq(EndTurn::build_sequence());
            } else {
                // Java: changePlayerAction(null, null, false) — no-op (null player)
                // Java: Inducement(END_OF_OWN_TURN, homePlaying)
                let seq = Inducement::build_sequence(&InducementParams {
                    inducement_phase: InducementPhase::EndOfOwnTurn,
                    home_team: game.home_playing,
                    check_forgo: false,
                });
                return StepOutcome::next().push_seq(seq);
            }
        }

        // Java: else if (!fEndPlayerAction && throwerAction != null && throwerAction.isPassing())
        let thrower_passing = game.thrower_action
            .map(|a| a.is_passing())
            .unwrap_or(false);
        if !self.end_player_action && thrower_passing {
            let seq = Pass::build_sequence(&PassParams {
                target_coordinate: game.pass_coordinate,
            });
            return StepOutcome::next().push_seq(seq);
        }

        // Java: else if (KickoffReturn || PassBlock) → publish EndPlayerAction
        if matches!(game.turn_mode, TurnMode::KickoffReturn | TurnMode::PassBlock) {
            return StepOutcome::next()
                .publish(StepParameter::EndPlayerAction(true));
        }

        // Java: game.setPassCoordinate(null)
        game.pass_coordinate = None;
        // Java: changePlayerAction(null, null, false) — no-op (null player)
        // Java: Select.pushSequence(false)
        let seq = Select::build_sequence(&SelectParams { update_persistence: false });
        StepOutcome::next().push_seq(seq)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepId};
    use ffb_model::enums::{Rules, TurnMode};

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    #[test]
    fn default_pushes_select_sequence() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        let mut step = StepEndFeeding::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(out.pushes.len(), 1);
        assert_eq!(out.pushes[0][0].step_id, StepId::InitSelecting);
    }

    #[test]
    fn end_turn_pass_block_pushes_end_turn_sequence() {
        let mut game = make_game();
        game.turn_mode = TurnMode::PassBlock;
        let mut step = StepEndFeeding::new();
        step.end_turn = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.pushes.len(), 1);
        assert_eq!(out.pushes[0][0].step_id, StepId::EndTurn);
    }

    #[test]
    fn end_turn_regular_pushes_inducement_end_of_own_turn() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        let mut step = StepEndFeeding::new();
        step.end_turn = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.pushes.len(), 1);
        assert_eq!(out.pushes[0][0].step_id, StepId::InitInducement);
    }

    #[test]
    fn kickoff_return_publishes_end_player_action() {
        let mut game = make_game();
        game.turn_mode = TurnMode::KickoffReturn;
        let mut step = StepEndFeeding::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndPlayerAction(true))));
    }

    #[test]
    fn pass_block_without_end_turn_publishes_end_player_action() {
        let mut game = make_game();
        game.turn_mode = TurnMode::PassBlock;
        let mut step = StepEndFeeding::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndPlayerAction(true))));
    }

    #[test]
    fn default_clears_pass_coordinate() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        game.pass_coordinate = Some(ffb_model::types::FieldCoordinate::new(5, 5));
        let mut step = StepEndFeeding::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.pass_coordinate.is_none());
    }

    #[test]
    fn set_parameter_end_player_action_accepted() {
        let mut step = StepEndFeeding::new();
        assert!(step.set_parameter(&StepParameter::EndPlayerAction(true)));
        assert!(step.end_player_action);
    }

    #[test]
    fn set_parameter_end_turn_accepted() {
        let mut step = StepEndFeeding::new();
        assert!(step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(step.end_turn);
    }

    #[test]
    fn unrecognised_parameter_returns_false() {
        let mut step = StepEndFeeding::new();
        assert!(!step.set_parameter(&StepParameter::CheckForgo(true)));
    }
}
