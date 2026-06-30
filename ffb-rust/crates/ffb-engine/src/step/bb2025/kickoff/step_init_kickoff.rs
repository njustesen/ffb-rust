use ffb_model::enums::{InducementPhase, TurnMode};
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::sequences::inducement_sequence;

/// Initialises the kickoff sequence.
///
/// Java logic:
///  1. If TurnMode is START_GAME → call stateMechanic.startHalf(1), set TurnMode::Setup,
///     startTurn(), prepareForSetup(). (Half-start bookkeeping.)
///  2. Push two Inducement sequences (InducementPhase::BEFORE_SETUP) for each team.
///  3. NEXT_STEP.
///
/// Rust: step 1 (`startHalf`, `prepareForSetup`) and step 2 (Inducement sequence generator)
/// are stubs — they require infrastructure not yet ported.  The TurnMode transition and
/// NEXT_STEP are implemented.
///
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.kickoff.StepInitKickoff`.
pub struct StepInitKickoff;

impl StepInitKickoff {
    pub fn new() -> Self { Self }
}

impl Default for StepInitKickoff {
    fn default() -> Self { Self::new() }
}

impl Step for StepInitKickoff {
    fn id(&self) -> StepId { StepId::InitKickoff }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

impl StepInitKickoff {
    fn execute_step(&self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        if game.turn_mode == TurnMode::StartGame {
            // Java: stateMechanic.startHalf(this, 1)
            game.half = 1;
            game.turn_data_home.turn_nr = 0;
            game.turn_data_away.turn_nr = 0;
            // Java: homePlaying = homeFirstOffense ? (half % 2 == 0) : (half % 2 > 0)
            game.home_playing = if game.home_first_offense { game.half % 2 == 0 } else { game.half % 2 > 0 };
            game.field_model.ball_coordinate = None;
            game.field_model.ball_in_play = false;
            // TODO: apothecaries, rerolls, leader state, special skills, inducements reset
            game.turn_mode = TurnMode::Setup;
            // TODO: startTurn(), prepareForSetup()
        }

        // Java: push Inducement(BEFORE_SETUP, !home_playing) then Inducement(BEFORE_SETUP, home_playing).
        let home = game.home_playing;
        let seq_opponent = inducement_sequence(InducementPhase::BeforeSetup, !home);
        let seq_own = inducement_sequence(InducementPhase::BeforeSetup, home);
        StepOutcome::next()
            .push_seq(seq_opponent)
            .push_seq(seq_own)
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
    fn start_returns_next_step_and_pushes_two_inducement_sequences() {
        let mut game = make_game();
        let mut step = StepInitKickoff::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(out.pushes.len(), 2);
    }

    #[test]
    fn start_game_mode_transitions_to_setup() {
        let mut game = make_game();
        game.turn_mode = TurnMode::StartGame;
        let mut step = StepInitKickoff::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.turn_mode, TurnMode::Setup);
        assert_eq!(game.half, 1);
        assert_eq!(game.turn_data_home.turn_nr, 0);
        assert_eq!(game.turn_data_away.turn_nr, 0);
    }

    #[test]
    fn non_start_game_mode_unchanged() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Kickoff;
        let mut step = StepInitKickoff::new();
        step.start(&mut game, &mut GameRng::new(0));
        // TurnMode should remain Kickoff (not transitioned to Setup)
        assert_eq!(game.turn_mode, TurnMode::Kickoff);
    }

    #[test]
    fn handle_command_returns_next_step() {
        let mut game = make_game();
        let mut step = StepInitKickoff::new();
        let out = step.handle_command(&Action::Acknowledge, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_returns_false() {
        let mut step = StepInitKickoff::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(false)));
    }
}
