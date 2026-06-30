/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2020.kickoff.StepSetup`.
///
/// Handles team setup before kickoff (BB2020).
///
/// Java logic on `fEndSetup = true`:
///  1. Play the "ding" sound.
///  2. Call `SetupMechanic.checkSetup(gameState, isHomePlaying)`.
///     - If valid:
///       - flip `home_playing`, reset turn data flags, refresh boxes.
///       - if `isSetupOffense`: set turn mode to KICKOFF.
///       - else: set `setup_offense = true`, push two Inducement(BEFORE_SETUP) sequences.
///     - If invalid: reset `fEndSetup = false`, stay (Continue).
///  3. CLIENT_TEAM_SETUP_LOAD/SAVE/DELETE and CLIENT_SETUP_PLAYER are SKIP_STEP.
///  4. CLIENT_END_TURN (from current player) sets `fEndSetup = true` and calls EXECUTE_STEP.
///
/// Mandatory init param: `GOTO_LABEL_ON_END` (used by the generator, stored here for JSON).
///
/// TODO(StepSetup-mechanic): SetupMechanic.checkSetup port deferred; always accepts for now.
/// TODO(StepSetup-boxes): UtilBox.refreshBoxes deferred.
/// TODO(StepSetup-turnmode): game.setTurnMode(TurnMode.KICKOFF) deferred.
/// TODO(StepSetup-teamsetup): CLIENT_TEAM_SETUP_LOAD/SAVE/DELETE deferred.
use ffb_model::enums::InducementPhase;
use ffb_model::model::game::Game;
use ffb_model::prompts::AgentPrompt;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::generator::common::Inducement;
use crate::step::generator::common::inducement::InducementParams;

/// Java: `StepSetup` (bb2020/kickoff).
pub struct StepSetup {
    /// Java: `fGotoLabelOnEnd` — mandatory init param.
    pub goto_label_on_end: String,
    /// Java: `fEndSetup` — set true when the coach sends CLIENT_END_TURN.
    pub end_setup: bool,
}

impl StepSetup {
    pub fn new() -> Self {
        Self {
            goto_label_on_end: String::new(),
            end_setup: false,
        }
    }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if !self.end_setup {
            // Java: implicit setup UI (the step stays in Continue waiting for the client).
            // Emit TeamSetup prompt so the driver waits. Auto-confirm for empty teams.
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
            return StepOutcome::cont()
                .with_prompt(AgentPrompt::TeamSetup { team_id, players: player_ids });
        }

        // Java: getResult().setSound(SoundId.DING)
        // Java: SetupMechanic.checkSetup(gameState, game.isHomePlaying())
        // TODO(StepSetup-mechanic): port SetupMechanic.checkSetup. Always accept for now.
        let setup_valid = true;

        if setup_valid {
            // Java: game.setHomePlaying(!game.isHomePlaying())
            game.home_playing = !game.home_playing;

            // Java: game.getTurnData().setTurnStarted(false)
            // Java: game.getTurnData().setFirstTurnAfterKickoff(false)
            game.turn_data_mut().turn_started = false;
            game.turn_data_mut().first_turn_after_kickoff = false;

            // Java: UtilBox.refreshBoxes(game)
            // TODO(StepSetup-boxes): port refreshBoxes.

            if game.setup_offense {
                // Java: game.setTurnMode(TurnMode.KICKOFF)
                // TODO(StepSetup-turnmode): set turn mode to KICKOFF here.
                // Java: getResult().setNextAction(StepAction.NEXT_STEP)
                self.end_setup = false;
                StepOutcome::next()
            } else {
                // Java: game.setSetupOffense(true)
                game.setup_offense = true;
                // Java: push Inducement(BEFORE_SETUP, home_playing) + Inducement(BEFORE_SETUP, !home_playing)
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
                // Java: getResult().setNextAction(StepAction.NEXT_STEP)
                StepOutcome::next()
                    .push_seq(seq_own)
                    .push_seq(seq_opponent)
            }
        } else {
            // Invalid setup: let the coach try again.
            self.end_setup = false;
            StepOutcome::cont()
        }
    }
}

impl Default for StepSetup {
    fn default() -> Self { Self::new() }
}

impl Step for StepSetup {
    fn id(&self) -> StepId { StepId::Setup }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: start() calls executeStep() (no super.start() call in bb2020 version)
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: commandStatus = super.handleCommand(pReceivedCommand)
        // Java: switch (pReceivedCommand.getId()):
        //   CLIENT_TEAM_SETUP_LOAD → UtilServerSetup.loadTeamSetup; SKIP_STEP
        //   CLIENT_TEAM_SETUP_SAVE → UtilServerSetup.saveTeamSetup; SKIP_STEP
        //   CLIENT_TEAM_SETUP_DELETE → UtilServerSetup.deleteTeamSetup; SKIP_STEP
        //   CLIENT_SETUP_PLAYER → UtilServerSetup.setupPlayer; SKIP_STEP
        //   CLIENT_END_TURN (from current player) → set fEndSetup=true; EXECUTE_STEP
        match action {
            Action::ConfirmSetup => {
                // Java: CLIENT_END_TURN → fEndSetup = true; EXECUTE_STEP
                self.end_setup = true;
            }
            Action::PlacePlayer { player_id, coord } => {
                // Java: CLIENT_SETUP_PLAYER → UtilServerSetup.setupPlayer(...); SKIP_STEP
                game.field_model.set_player_coordinate(player_id, *coord);
                return StepOutcome::cont();
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            // Java: case GOTO_LABEL_ON_END: fGotoLabelOnEnd = (String) parameter.getValue()
            StepParameter::GotoLabelOnEnd(v) => {
                self.goto_label_on_end = v.clone();
                true
            }
            _ => false,
        }
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::Rules;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    #[test]
    fn id_is_setup() {
        assert_eq!(StepSetup::new().id(), StepId::Setup);
    }

    #[test]
    fn start_with_no_players_auto_advances() {
        // Zero-player teams: setup is trivially valid, auto-advance.
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
    fn set_parameter_goto_label_on_end() {
        let mut step = StepSetup::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnEnd("END".into())));
        assert_eq!(step.goto_label_on_end, "END");
    }

    #[test]
    fn place_player_returns_cont_without_advancing() {
        let mut game = make_game();
        let mut step = StepSetup::new();
        let coord = ffb_model::types::FieldCoordinate::new(5, 7);
        let out = step.handle_command(
            &Action::PlacePlayer { player_id: "p1".into(), coord },
            &mut game,
            &mut GameRng::new(0),
        );
        assert_eq!(out.action, StepAction::Continue);
        assert_eq!(game.field_model.player_coordinate("p1"), Some(coord));
    }

    #[test]
    fn first_offense_setup_pushes_two_inducement_sequences() {
        let mut game = make_game();
        game.setup_offense = false;
        let mut step = StepSetup::new();
        let out = step.handle_command(&Action::ConfirmSetup, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(out.pushes.len(), 2, "should push two BeforeSetup Inducement sequences");
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
    fn subsequent_setup_no_sequence_push() {
        let mut game = make_game();
        game.setup_offense = true;
        let mut step = StepSetup::new();
        let out = step.handle_command(&Action::ConfirmSetup, &mut game, &mut GameRng::new(0));
        assert!(out.pushes.is_empty(), "no sequences after first offense setup");
    }

    #[test]
    fn turn_data_reset_on_confirm() {
        let mut game = make_game();
        game.turn_data_mut().turn_started = true;
        game.turn_data_mut().first_turn_after_kickoff = true;
        let mut step = StepSetup::new();
        step.handle_command(&Action::ConfirmSetup, &mut game, &mut GameRng::new(0));
        // The turn data for the flipped side is reset.
        // Since home_playing flips, we check via the turn data accessor for the now-active side.
        // Both were reset before the flip.
        assert!(!game.turn_data_mut().turn_started);
        assert!(!game.turn_data_mut().first_turn_after_kickoff);
    }
}
