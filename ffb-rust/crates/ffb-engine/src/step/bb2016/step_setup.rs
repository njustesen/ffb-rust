use ffb_model::enums::TurnMode;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2016.StepSetup.
///
/// Handles team setup before kickoff. Manages loading/saving/deleting presets
/// (SKIP_STEP), individual player placement (SKIP_STEP), and the final
/// CLIENT_END_TURN that commits the setup.
///
/// On end-setup:
///   - checkNoPlayersInBoxOrField → if triggered, goto GOTO_LABEL_ON_END (auto-TD)
///   - checkSetup() validates formation rules
///   - toggles isHomePlaying, pushes Inducement sequences for BEFORE_SETUP phase
///   - or switches to KICKOFF turn mode
///
/// Init: mandatory GOTO_LABEL_ON_END.
pub struct StepSetup {
    /// Java: fGotoLabelOnEnd (mandatory)
    pub goto_label_on_end: Option<String>,
    /// Java: fEndSetup
    pub end_setup: bool,
}

impl StepSetup {
    pub fn new() -> Self {
        Self {
            goto_label_on_end: None,
            end_setup: false,
        }
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
            // Java: CLIENT_TEAM_SETUP_LOAD/SAVE/DELETE, CLIENT_SETUP_PLAYER → SKIP_STEP
            // In Rust: these are no-ops (the engine processes them transparently)
            Action::PlacePlayer { player_id, coord } => {
                // Java: UtilServerSetup.setupPlayer(getGameState(), playerId, coordinate)
                // TODO: UtilServerSetup.setupPlayer
                let _ = (player_id, coord);
                return StepOutcome::cont(); // SKIP_STEP → stay in setup
            }
            Action::ConfirmSetup => {
                // Java: CLIENT_END_TURN from current player → fEndSetup = true → executeStep
                self.end_setup = true;
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnEnd(label) => {
                self.goto_label_on_end = Some(label.clone());
                true
            }
            _ => false,
        }
    }
}

impl StepSetup {
    fn execute_step(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        let goto_label = match &self.goto_label_on_end {
            Some(l) => l.clone(),
            None => return StepOutcome::cont(), // misconfigured
        };

        // Java: checkNoPlayersInBoxOrField()
        if self.check_no_players_in_box_or_field(game) {
            game.turn_mode = TurnMode::NoPlayersToField;
            return StepOutcome::goto(&goto_label);
        }

        if self.end_setup {
            // Java: getResult().setSound(SoundId.DING)
            // Java: SetupMechanic.checkSetup(getGameState(), game.isHomePlaying())
            // TODO: SetupMechanic.checkSetup — validate formation
            // Stub: assume setup is always valid
            let setup_valid = true;
            if setup_valid {
                game.home_playing = !game.home_playing;
                // Java: game.getTurnData().setTurnStarted(false)
                // Java: game.getTurnData().setFirstTurnAfterKickoff(false)
                // Java: UtilBox.refreshBoxes(game)
                // TODO: box refresh, turn data updates

                if game.setup_offense {
                    // Java: game.setTurnMode(TurnMode.KICKOFF)
                    game.turn_mode = TurnMode::Kickoff;
                } else {
                    // Java: game.setSetupOffense(true)
                    game.setup_offense = true;
                    // Java: push Inducement sequences for BEFORE_SETUP phase (home + away)
                    // TODO: SequenceGeneratorFactory.Inducement.pushSequence(...)
                }
                return StepOutcome::next();
            } else {
                self.end_setup = false;
                return StepOutcome::cont(); // back to setup
            }
        }

        // Java: if neither trigger fired → wait for client commands
        StepOutcome::cont()
    }

    /// Java: checkNoPlayersInBoxOrField() — awards TD if one team has no eligible players.
    fn check_no_players_in_box_or_field(&self, game: &mut Game) -> bool {
        // Java: findPlayersInReserveOrField(game, teamHome) / (game, teamAway)
        // TODO: actual player count lookup via field model
        // Stub: always return false (no auto-TD)
        let _ = game;
        false
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
        Game::new(home, away, Rules::Bb2016)
    }

    #[test]
    fn step_id_is_setup() {
        let step = StepSetup::new();
        assert_eq!(step.id(), StepId::Setup);
    }

    #[test]
    fn goto_label_on_end_parameter_accepted() {
        let mut step = StepSetup::new();
        let ok = step.set_parameter(&StepParameter::GotoLabelOnEnd("end".to_string()));
        assert!(ok);
        assert_eq!(step.goto_label_on_end.as_deref(), Some("end"));
    }

    #[test]
    fn start_without_label_returns_cont() {
        let mut step = StepSetup::new();
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        // No label configured → cont (misconfigured guard)
        assert_eq!(out.action, StepAction::Continue);
    }

    #[test]
    fn start_with_label_waits_for_client() {
        let mut step = StepSetup::new();
        step.goto_label_on_end = Some("end".to_string());
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Not end_setup yet → Continue
        assert_eq!(out.action, StepAction::Continue);
    }

    #[test]
    fn confirm_setup_transitions_to_next() {
        let mut step = StepSetup::new();
        step.goto_label_on_end = Some("end".to_string());
        let mut game = make_game();
        let out = step.handle_command(&Action::ConfirmSetup, &mut game, &mut GameRng::new(0));
        // Stub: setup always valid → NextStep
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn confirm_setup_toggles_home_playing() {
        let mut step = StepSetup::new();
        step.goto_label_on_end = Some("end".to_string());
        let mut game = make_game();
        game.home_playing = true;
        step.handle_command(&Action::ConfirmSetup, &mut game, &mut GameRng::new(0));
        assert!(!game.home_playing);
    }

    #[test]
    fn confirm_setup_sets_kickoff_when_setup_offense() {
        let mut step = StepSetup::new();
        step.goto_label_on_end = Some("end".to_string());
        let mut game = make_game();
        game.setup_offense = true;
        step.handle_command(&Action::ConfirmSetup, &mut game, &mut GameRng::new(0));
        assert_eq!(game.turn_mode, TurnMode::Kickoff);
    }

    #[test]
    fn unrecognised_parameter_returns_false() {
        let mut step = StepSetup::new();
        let accepted = step.set_parameter(&StepParameter::EndTurn(true));
        assert!(!accepted);
    }
}
