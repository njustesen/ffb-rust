use ffb_model::enums::TurnMode;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, SequenceStep};
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::bb2016::Select;
use crate::step::generator::bb2016::select::SelectParams;

/// Executes the Charge/Blitz! kickoff result turn (BB2016).
///
/// Two-entry state machine driven by TurnMode:
///  - Entry 1 (non-BLITZ): pin players, set TurnMode=BLITZ, push self + Select onto stack.
///  - Entry 2 (BLITZ): if fEndTurn → set TurnMode=KICKOFF; NEXT_STEP.
///
/// `pinPlayersInTacklezones` and `startTurn` are TODO stubs.
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2016.StepBlitzTurn`.
pub struct StepBlitzTurn {
    /// Java: fEndTurn
    pub end_turn: bool,
}

impl StepBlitzTurn {
    pub fn new() -> Self { Self { end_turn: false } }
}

impl Default for StepBlitzTurn {
    fn default() -> Self { Self::new() }
}

impl Step for StepBlitzTurn {
    fn id(&self) -> StepId { StepId::BlitzTurn }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::EndTurn(v) => { self.end_turn = *v; true }
            _ => false,
        }
    }
}

impl StepBlitzTurn {
    fn execute_step(&self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        if game.turn_mode == TurnMode::Blitz {
            // Second entry: blitz turn is over.
            if self.end_turn {
                game.turn_mode = TurnMode::Kickoff;
            }
            StepOutcome::next()
        } else {
            // First entry: set up the blitz turn.
            // DEFERRED(SetupMechanic): pinPlayersInTacklezones not yet ported
            game.turn_mode = TurnMode::Blitz;
            // DEFERRED(timer): UtilServerTimer.stopTurnTimer / startTurnTimer not yet ported
            // DEFERRED(startTurn): game.startTurn() not yet ported

            // Java: pushCurrentStepOnStack() — push self back; then push Select on top.
            // Push order: self first (bottom), Select on top (runs first).
            let self_seq = vec![SequenceStep::new(StepId::BlitzTurn)];
            let select_seq = Select::build_sequence(&SelectParams { update_persistence: true });
            // DEFERRED(UtilServerGame): updatePlayerStateDependentProperties not yet ported
            StepOutcome::next()
                .push_seq(self_seq)
                .push_seq(select_seq)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::Rules;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    #[test]
    fn first_entry_sets_blitz_mode() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Kickoff;
        let mut step = StepBlitzTurn::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.turn_mode, TurnMode::Blitz);
    }

    #[test]
    fn first_entry_pushes_self_and_select_sequence() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Kickoff;
        let mut step = StepBlitzTurn::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.pushes.len(), 2);
        assert_eq!(out.pushes[0][0].step_id, StepId::BlitzTurn);
        assert_eq!(out.pushes[1][0].step_id, StepId::InitSelecting);
    }

    #[test]
    fn second_entry_end_turn_sets_kickoff_mode() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Blitz;
        let mut step = StepBlitzTurn::new();
        step.end_turn = true;
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.turn_mode, TurnMode::Kickoff);
    }

    #[test]
    fn second_entry_no_end_turn_stays_in_blitz() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Blitz;
        let mut step = StepBlitzTurn::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.turn_mode, TurnMode::Blitz);
    }

    #[test]
    fn always_returns_next_step() {
        let mut game = make_game();
        let mut step = StepBlitzTurn::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_end_turn_accepted() {
        let mut step = StepBlitzTurn::new();
        assert!(step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(step.end_turn);
    }

    #[test]
    fn set_parameter_unknown_returns_false() {
        let mut step = StepBlitzTurn::new();
        assert!(!step.set_parameter(&StepParameter::HomeTeam(true)));
    }
}
