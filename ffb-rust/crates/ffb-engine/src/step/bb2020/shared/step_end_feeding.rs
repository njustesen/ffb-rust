use ffb_model::enums::{InducementPhase, TurnMode};
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::bb2020::pass::{Pass, PassParams};
use crate::step::generator::bb2020::select::{Select, SelectParams};
use crate::step::generator::common::inducement::{Inducement, InducementParams};
use crate::step::generator::mixed::end_turn::EndTurn;
use crate::step::generator::sequence::{Sequence, SequenceStep};
use crate::step::util_server_steps::check_touchdown;

/// Final step of the feed sequence. Consumes EndPlayerAction/EndTurn.
/// 1:1 translation of com.fumbbl.ffb.server.step.bb2020.shared.StepEndFeeding.
///
/// BB2020 differs from BB2025: no CheckForgo parameter.
/// Full logic requires sequence generators.
pub struct StepEndFeeding {
    pub end_player_action: bool,
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
        self.end_turn |= check_touchdown(game);
        game.defender_id = None;

        if self.end_turn {
            if game.turn_mode == TurnMode::PassBlock {
                // Java: EndTurn.pushSequence(getGameState(), false)
                return StepOutcome::next().push_seq(EndTurn::build_sequence());
            } else {
                // Java: UtilServerSteps.changePlayerAction(this, null, null, false) → clear acting player
                game.acting_player.player_id = None;
            }

            if game.turn_mode == TurnMode::Regular {
                // Java: Inducement(END_OF_OPPONENT_TURN, !home_playing) + Inducement(END_OF_OWN_TURN, home_playing) + PickMeUp
                let seq_opp = Inducement::build_sequence(&InducementParams {
                    inducement_phase: InducementPhase::EndOfOpponentTurn,
                    home_team: !game.home_playing,
                    check_forgo: false,
                });
                let seq_own = Inducement::build_sequence(&InducementParams {
                    inducement_phase: InducementPhase::EndOfOwnTurn,
                    home_team: game.home_playing,
                    check_forgo: false,
                });
                let mut seq_pick = Sequence::new();
                seq_pick.add(StepId::PickMeUp, vec![]);
                return StepOutcome::next()
                    .push_seq(seq_opp)
                    .push_seq(seq_own)
                    .push_seq(seq_pick.build());
            } else if game.turn_mode == TurnMode::KickoffReturn {
                return StepOutcome::next().push_seq(EndTurn::build_sequence());
            }
        } else if !self.end_player_action {
            // Check if thrower action is passing
            let is_passing = game.thrower_action.map(|a| a.is_passing()).unwrap_or(false);
            if is_passing {
                let coord = game.pass_coordinate;
                return StepOutcome::next().push_seq(Pass::build_sequence(&PassParams {
                    target_coordinate: coord,
                }));
            } else if game.turn_mode == TurnMode::KickoffReturn || game.turn_mode == TurnMode::PassBlock {
                // Java: publishParameter(END_PLAYER_ACTION, true)
                return StepOutcome::next().publish(StepParameter::EndPlayerAction(true));
            } else {
                game.pass_coordinate = None;
                // Java: UtilServerSteps.changePlayerAction(this, null, null, false) → clear acting player
                game.acting_player.player_id = None;
                return StepOutcome::next().push_seq(Select::build_sequence(&SelectParams::default()));
            }
        }

        StepOutcome::next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::Rules;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2020)
    }

    #[test]
    fn set_parameter_end_turn_accepted() {
        let mut step = StepEndFeeding::new();
        assert!(step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(step.end_turn);
    }

    #[test]
    fn set_parameter_end_player_action_accepted() {
        let mut step = StepEndFeeding::new();
        assert!(step.set_parameter(&StepParameter::EndPlayerAction(true)));
        assert!(step.end_player_action);
    }

    #[test]
    fn start_returns_next() {
        let mut game = make_game();
        let mut step = StepEndFeeding::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn no_check_forgo_parameter() {
        // BB2020 StepEndFeeding does not accept CheckForgo (unlike BB2025).
        let mut step = StepEndFeeding::new();
        assert!(!step.set_parameter(&StepParameter::CheckForgo(true)));
    }

    #[test]
    fn end_turn_pass_block_pushes_end_turn_sequence() {
        let mut game = make_game();
        game.turn_mode = TurnMode::PassBlock;
        let mut step = StepEndFeeding::new();
        step.end_turn = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(!out.pushes.is_empty());
    }

    #[test]
    fn end_turn_regular_pushes_three_sequences() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        let mut step = StepEndFeeding::new();
        step.end_turn = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.pushes.len(), 3);
    }

    #[test]
    fn default_pushes_select_sequence() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        let mut step = StepEndFeeding::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(!out.pushes.is_empty());
    }

    #[test]
    fn clears_defender_id_on_execute() {
        let mut game = make_game();
        game.defender_id = Some("p1".into());
        game.turn_mode = TurnMode::Regular;
        let mut step = StepEndFeeding::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.defender_id.is_none());
    }
}
