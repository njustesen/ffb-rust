use ffb_model::enums::{InducementPhase, TurnMode};
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::report::report_start_half::ReportStartHalf;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::mechanic::bb2025::state_mechanic::StateMechanic;
use crate::mechanic::state_mechanic::StateMechanic as StateMechanicTrait;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::sequences::inducement_sequence;
use crate::util::util_server_game::UtilServerGame;

/// Initialises the kickoff sequence.
///
/// Java logic:
///  1. If TurnMode is START_GAME → call stateMechanic.startHalf(1), set TurnMode::Setup,
///     startTurn(), prepareForSetup(). (Half-start bookkeeping.)
///  2. Push two Inducement sequences (InducementPhase::BEFORE_SETUP) for each team.
///  3. NEXT_STEP.
///
/// Rust: step 1 fully implemented (`startHalf`, `update_player_state_dependent_properties`,
/// `prepare_for_setup`). Step 2 (Inducement sequence generator) wired. The TurnMode transition and
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
        let mut events: Vec<GameEvent> = Vec::new();
        if game.turn_mode == TurnMode::StartGame {
            // Java: stateMechanic.startHalf(step, 1) — returns inducement-registration events
            let half_events = StateMechanic::new().start_half(game, 1);
            events.extend(half_events);
            // Java: getResult().addReport(new ReportStartHalf(game.getHalf()))
            game.report_list.add(ReportStartHalf::new(game.half));
            events.push(GameEvent::StartHalf { half: game.half });
            game.turn_mode = TurnMode::Setup;
            game.start_turn();
            // Java: com.fumbbl.ffb.server.step.bb2025.kickoff.StepInitKickoff.executeStep does NOT call
            // UtilServerGame.updatePlayerStateDependentProperties (unlike the mixed BB2016/BB2020
            // StepInitKickoff, which does). Only prepareForSetup is called here.
            UtilServerGame::prepare_for_setup(game);
        }

        // Java: push Inducement(BEFORE_SETUP, !home_playing) then Inducement(BEFORE_SETUP, home_playing).
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
    fn start_game_mode_transitions_to_setup_and_calls_start_half() {
        let mut game = make_game();
        game.team_home.rerolls = 2;
        game.team_away.rerolls = 1;
        game.turn_mode = TurnMode::StartGame;
        let mut step = StepInitKickoff::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.turn_mode, TurnMode::Setup);
        assert_eq!(game.half, 1);
        assert_eq!(game.turn_data_home.turn_nr, 0);
        assert_eq!(game.turn_data_away.turn_nr, 0);
        // start_half wires up rerolls from team
        assert_eq!(game.turn_data_home.rerolls, 2);
        assert_eq!(game.turn_data_away.rerolls, 1);
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

    #[test]
    fn start_game_mode_adds_start_half_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        game.team_home.rerolls = 1;
        game.team_away.rerolls = 1;
        game.turn_mode = TurnMode::StartGame;
        let mut step = StepInitKickoff::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::START_HALF),
            "START_HALF report must be added when StartGame mode");
    }

    /// Java's bb2025 StepInitKickoff.executeStep does NOT call
    /// UtilServerGame.updatePlayerStateDependentProperties (only the mixed BB2016/BB2020
    /// StepInitKickoff does). Regression test: single_use_rerolls must not be recomputed
    /// (and thus overwritten) by the StartGame transition.
    #[test]
    fn start_game_mode_does_not_recompute_single_use_rerolls() {
        let mut game = make_game();
        game.turn_mode = TurnMode::StartGame;
        game.turn_data_home.single_use_rerolls = 99;
        game.turn_data_away.single_use_rerolls = 99;
        let mut step = StepInitKickoff::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.turn_data_home.single_use_rerolls, 99);
        assert_eq!(game.turn_data_away.single_use_rerolls, 99);
    }

    #[test]
    fn non_start_game_mode_does_not_add_start_half_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        game.turn_mode = TurnMode::Kickoff;
        let mut step = StepInitKickoff::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(!game.report_list.has_report(ReportId::START_HALF),
            "START_HALF report must NOT be added when not StartGame mode");
    }
}
