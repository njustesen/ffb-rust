use ffb_model::enums::TurnMode;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::mechanic::mixed::setup_mechanic::SetupMechanic;
use crate::mechanic::setup_mechanic::SetupMechanic as SetupMechanicTrait;
use crate::step::framework::{Step, StepOutcome, SequenceStep};
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::bb2025::Select;
use crate::step::generator::bb2025::select::SelectParams;

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
///     - game.startTurn() — now implemented.
///     - Push `this` step back onto the stack (so we re-enter on the second call).
///     - Push a Select sequence for the blitzing team.
///     - NEXT_STEP.
///
/// TurnMode transitions, `pinPlayersInTacklezones`, `startTurn()`, and Select push all implemented.
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
            let blitzing_team_id = if game.home_playing {
                game.team_home.id.clone()
            } else {
                game.team_away.id.clone()
            };
            SetupMechanic::new().pin_players_in_tacklezones_chain(game, &blitzing_team_id, true);
            // no-op: headless engine has no turn timer; stopTurnTimer/startTurnTimer are server-only
            game.start_turn();
            game.turn_mode = TurnMode::Blitz;
            // Java: pushCurrentStepOnStack(); Select.pushSequence(gameState, true)
            let self_seq = vec![SequenceStep::new(StepId::BlitzTurn)];
            let select_seq = Select::build_sequence(&SelectParams {
                update_persistence: true,
                is_blitz_move: false,
                ..Default::default()
            });
            return StepOutcome::next().push_seq(self_seq).push_seq(select_seq);
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

    #[test]
    fn first_entry_pushes_self_seq_and_select_seq() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Kickoff;
        let mut step = StepBlitzTurn::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Java: pushCurrentStepOnStack() + Select.pushSequence → two pushed sequences
        assert_eq!(out.pushes.len(), 2, "must push self_seq + select_seq");
        assert_eq!(out.pushes[0][0].step_id, StepId::BlitzTurn);
    }
}
