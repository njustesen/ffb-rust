use ffb_model::enums::InducementPhase;
use ffb_model::model::game::Game;
use ffb_model::prompts::AgentPrompt;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::common::Inducement;
use crate::step::generator::common::inducement::InducementParams;

/// Handles team setup (pre-kickoff player placement).
///
/// Java logic on `fEndSetup = true`:
///  1. Play the "ding" sound.
///  2. Call `SetupMechanic.checkSetup(gameState, isHomePlaying)`.
///     - If valid: flip `home_playing`, reset turn data flags, refresh boxes.
///     - If it's the first offense setup (`!isSetupOffense`): push inducement
///       sequences (InducementPhase::BEFORE_SETUP) for both teams.
///     - NEXT_STEP.
///  3. If invalid: set `fEndSetup = false` and stay (Continue).
///
/// The setup validity check (`checkSetup`) and box refresh are TODO stubs.
/// The TurnData reset and `home_playing` flip are implemented.
///
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.kickoff.StepSetup`.
pub struct StepSetup {
    /// Java: fEndSetup — set true when the coach sends CLIENT_END_TURN.
    pub end_setup: bool,
}

impl StepSetup {
    pub fn new() -> Self {
        Self { end_setup: false }
    }
}

impl Default for StepSetup {
    fn default() -> Self { Self::new() }
}

impl Step for StepSetup {
    fn id(&self) -> StepId { StepId::Setup }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::ConfirmSetup => {
                self.end_setup = true;
            }
            Action::PlacePlayer { player_id, coord } => {
                // Java CLIENT_SETUP_PLAYER → UtilServerSetup.setupPlayer(...)
                // Place or remove the player at the given coordinate.
                game.field_model.set_player_coordinate(player_id, *coord);
                // Placement commands are SKIP_STEP in Java (no execute_step call).
                return StepOutcome::cont();
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

impl StepSetup {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if !self.end_setup {
            // Java: the client shows setup UI automatically (implicit dialog).
            // In Rust we must be explicit: emit a TeamSetup prompt so the driver waits.
            // Exception: if the team has no players, auto-confirm (trivially valid setup).
            let (team_id, player_ids) = if game.home_playing {
                (game.team_home.id.clone(), game.team_home.players.iter().map(|p| p.id.clone()).collect::<Vec<_>>())
            } else {
                (game.team_away.id.clone(), game.team_away.players.iter().map(|p| p.id.clone()).collect::<Vec<_>>())
            };
            if player_ids.is_empty() {
                // No players: auto-advance without waiting for a client command.
                self.end_setup = true;
                return self.execute_step(game, rng);
            }
            return StepOutcome::cont().with_prompt(AgentPrompt::TeamSetup { team_id, players: player_ids });
        }

        // Java: SetupMechanic.checkSetup — validate placement counts, LOS, etc.
        // DEFERRED: port SetupMechanic.checkSetup.  For now always accept.
        let setup_valid = true;

        if setup_valid {
            // Java: flip home_playing so the other team sets up next.
            game.home_playing = !game.home_playing;

            // Java: game.getTurnData().setTurnStarted(false); setFirstTurnAfterKickoff(false).
            game.turn_data_mut().turn_started = false;
            game.turn_data_mut().first_turn_after_kickoff = false;

            // Java: UtilBox.refreshBoxes(game) — reconcile dugout box contents.
            // DEFERRED: port refreshBoxes.

            if !game.setup_offense {
                // Java: push Inducement(BEFORE_SETUP, home_playing) + Inducement(BEFORE_SETUP, !home_playing)
                game.setup_offense = true;
                let seq_own = Inducement::build_sequence(&InducementParams {
                    inducement_phase: InducementPhase::BeforeSetup,
                    home_team: game.home_playing,
                    check_forgo: false,
                });
                let seq_opponent = Inducement::build_sequence(&InducementParams {
                    inducement_phase: InducementPhase::BeforeSetup,
                    home_team: !game.home_playing,
                    check_forgo: false,
                });
                return StepOutcome::next()
                    .push_seq(seq_own)
                    .push_seq(seq_opponent);
            }

            self.end_setup = false;
            StepOutcome::next()
        } else {
            // Invalid setup: stay and let the coach try again.
            self.end_setup = false;
            StepOutcome::cont()
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
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn start_with_no_players_auto_advances() {
        // Zero-player teams: setup is trivially valid, auto-advance without prompting.
        let mut game = make_game();
        let mut step = StepSetup::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn confirm_setup_action_returns_next_step() {
        let mut game = make_game();
        let mut step = StepSetup::new();
        let out = step.handle_command(&Action::ConfirmSetup, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn confirm_setup_flips_home_playing() {
        let mut game = make_game();
        game.home_playing = true;
        let mut step = StepSetup::new();
        step.handle_command(&Action::ConfirmSetup, &mut game, &mut GameRng::new(0));
        assert!(!game.home_playing);
    }

    #[test]
    fn place_player_action_returns_cont_without_advancing() {
        let mut game = make_game();
        let mut step = StepSetup::new();
        let coord = ffb_model::types::FieldCoordinate::new(5, 7);
        let out = step.handle_command(
            &Action::PlacePlayer { player_id: "p1".into(), coord },
            &mut game, &mut GameRng::new(0),
        );
        assert_eq!(out.action, StepAction::Continue);
        // Player should be placed at the coordinate.
        assert_eq!(game.field_model.player_coordinate("p1"), Some(coord));
    }

    #[test]
    fn first_offense_setup_sets_setup_offense_flag() {
        let mut game = make_game();
        game.setup_offense = false;
        let mut step = StepSetup::new();
        step.handle_command(&Action::ConfirmSetup, &mut game, &mut GameRng::new(0));
        assert!(game.setup_offense);
    }

    #[test]
    fn first_offense_setup_pushes_two_inducement_sequences() {
        let mut game = make_game();
        game.setup_offense = false;
        let mut step = StepSetup::new();
        let out = step.handle_command(&Action::ConfirmSetup, &mut game, &mut GameRng::new(0));
        assert_eq!(out.pushes.len(), 2, "should push two BeforeSetup Inducement sequences");
        assert_eq!(out.pushes[0][0].step_id, StepId::InitInducement);
        assert_eq!(out.pushes[1][0].step_id, StepId::InitInducement);
    }

    #[test]
    fn subsequent_offense_setup_no_sequence_push() {
        let mut game = make_game();
        game.setup_offense = true;
        let mut step = StepSetup::new();
        let out = step.handle_command(&Action::ConfirmSetup, &mut game, &mut GameRng::new(0));
        assert!(out.pushes.is_empty(), "no sequences after first offense");
    }
}
