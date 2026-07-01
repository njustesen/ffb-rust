use ffb_model::enums::TurnMode;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// Executes the Charge/Blitz! kickoff result turn.
///
/// Java logic:
///  1. If TurnMode is already BLITZ (second entry, after the blitz turn completes):
///     set TurnMode back to KICKOFF → NEXT_STEP.
///  2. Otherwise (first entry):
///     - Find the blitzing team (home_playing ? home : away).
///     - Call SetupMechanic.pinPlayersInTacklezones to deactivate pinned players.
///     - Set TurnMode = BLITZ.
///     - Start the blitz turn timer (TODO — timer infrastructure not ported).
///     - game.startTurn() (TODO — TurnData.startTurn not yet ported).
///     - Push `this` step back onto the stack (so we re-enter on the second call).
///     - Push a Select sequence for the blitzing team (TODO — generator not wired here).
///     - NEXT_STEP.
///
/// The `pinPlayersInTacklezones` call, `startTurn()`, and the Select-sequence push
/// are TODO stubs.  The TurnMode transitions are implemented.
///
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.kickoff.StepBlitzTurn`.
pub struct StepBlitzTurn;

impl StepBlitzTurn {
    pub fn new() -> Self { Self }
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

    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

impl StepBlitzTurn {
    fn execute_step(&self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        if game.turn_mode == TurnMode::Blitz {
            // Second entry: blitz turn is over, return to kickoff.
            game.turn_mode = TurnMode::Kickoff;
        } else {
            // First entry: set up the blitz turn for the kicking team.
            // DEFERRED: SetupMechanic.pinPlayersInTacklezones(gameState, blitzingTeam, true)
            // DEFERRED: UtilServerTimer.stopTurnTimer / startTurnTimer
            // DEFERRED: game.startTurn()
            game.turn_mode = TurnMode::Blitz;
            // DEFERRED: push self back onto the stack (StepStack::pushCurrentStep).
            // DEFERRED: push Select sequence for the blitzing team via SequenceGenerator::Select.
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
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
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
    fn second_entry_restores_kickoff_mode() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Blitz;
        let mut step = StepBlitzTurn::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.turn_mode, TurnMode::Kickoff);
    }

    #[test]
    fn start_returns_next_step() {
        let mut game = make_game();
        let mut step = StepBlitzTurn::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn handle_command_returns_next_step() {
        let mut game = make_game();
        let mut step = StepBlitzTurn::new();
        let out = step.handle_command(&Action::Acknowledge, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_returns_false() {
        let mut step = StepBlitzTurn::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(false)));
    }
}
