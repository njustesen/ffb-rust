use ffb_model::enums::{InducementPhase, TurnMode};
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::mechanic::mixed::state_mechanic::StateMechanic;
use crate::mechanic::state_mechanic::StateMechanic as StateMechanicTrait;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::sequences::inducement_sequence;
use crate::util::util_server_game::UtilServerGame;

/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.kickoff.StepInitKickoff`.
///
/// Initialises the kickoff sequence. If the game has not yet started
/// (`TurnMode::StartGame`), transitions into `TurnMode::Setup` and starts
/// the first half. Then pushes inducement sequences for both teams before
/// the setup phase.
///
/// BB2016 / BB2020.
pub struct StepInitKickoff;

impl StepInitKickoff {
    pub fn new() -> Self { Self }
}

impl Default for StepInitKickoff {
    fn default() -> Self { Self::new() }
}

impl Step for StepInitKickoff {
    fn id(&self) -> StepId { StepId::InitKickoff }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

impl StepInitKickoff {
    fn execute_step(&self, game: &mut Game) -> StepOutcome {
        let mut events: Vec<GameEvent> = Vec::new();
        if game.turn_mode == TurnMode::StartGame {
            // Java: stateMechanic.startHalf(step, 1) — returns inducement-registration events
            let half_events = StateMechanic::new().start_half(game, 1);
            events.extend(half_events);
            // Java: getResult().addReport(new ReportStartHalf(game.getHalf()))
            events.push(GameEvent::StartHalf { half: game.half });
            game.turn_mode = TurnMode::Setup;
            game.start_turn();
            UtilServerGame::update_player_state_dependent_properties(game);
            UtilServerGame::prepare_for_setup(game);
        }
        let home = game.home_playing;
        let seq_opponent = inducement_sequence(InducementPhase::BeforeSetup, !home);
        let seq_own = inducement_sequence(InducementPhase::BeforeSetup, home);
        StepOutcome::next()
            .push_seq(seq_opponent)
            .push_seq(seq_own)
            .with_events(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::model::game::Game;
    use ffb_model::enums::Rules;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    #[test]
    fn id_is_init_kickoff() {
        assert_eq!(StepInitKickoff::new().id(), StepId::InitKickoff);
    }

    #[test]
    fn start_returns_next_step_and_pushes_two_inducement_sequences() {
        let mut step = StepInitKickoff::new();
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(out.pushes.len(), 2);
    }

    #[test]
    fn start_game_mode_transitions_to_setup_and_calls_start_half() {
        let mut step = StepInitKickoff::new();
        let mut game = make_game();
        game.team_home.rerolls = 3;
        game.team_away.rerolls = 2;
        game.turn_mode = TurnMode::StartGame;
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.turn_mode, TurnMode::Setup);
        assert_eq!(game.half, 1);
        // start_half sets rerolls from team
        assert_eq!(game.turn_data_home.rerolls, 3);
        assert_eq!(game.turn_data_away.rerolls, 2);
    }

    #[test]
    fn non_start_game_mode_unchanged() {
        let mut step = StepInitKickoff::new();
        let mut game = make_game();
        game.turn_mode = TurnMode::Kickoff;
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.turn_mode, TurnMode::Kickoff);
    }

    #[test]
    fn set_parameter_returns_false() {
        let mut step = StepInitKickoff::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(false)));
    }
}
