/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2025.shared.StepForgoneStalling`.
///
/// Checks and handles the Stalling rule (deliberate time-wasting near the end-zone).
/// Runs during end-of-turn when the active player may have forgone scoring.
///
/// Gates (Java `start()`):
///   1. Turn mode must be REGULAR.
///   2. `check_forgo` parameter must be true (set by INIT_FOULING via CHECK_FORGO parameter).
///   3. Game option `enableStallingCheck` must be enabled.
///   4. An active ball-carrier must exist on the field.
///   5. `StallingExtension.is_considered_stalling()` must return true.
///
/// If all gates pass, logs the event and delegates to `StallingExtension.handle_staller()`.
use ffb_model::enums::TurnMode;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_player::UtilPlayer;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use super::stalling_extension::StallingExtension;

pub struct StepForgoneStalling {
    /// Java: checkForgo
    pub check_forgo: bool,
    stalling_extension: StallingExtension,
}

impl StepForgoneStalling {
    pub fn new() -> Self {
        Self { check_forgo: false, stalling_extension: StallingExtension::new() }
    }
}

impl Default for StepForgoneStalling {
    fn default() -> Self { Self::new() }
}

impl Step for StepForgoneStalling {
    fn id(&self) -> StepId { StepId::ForgoneStalling }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::CheckForgo(v) => { self.check_forgo = *v; true }
            _ => false,
        }
    }
}

impl StepForgoneStalling {
    fn execute_step(&self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: game.getTurnMode() == TurnMode.REGULAR && checkForgo &&
        //       UtilGameOption.isOptionEnabled(game, GameOptionId.ENABLE_STALLING_CHECK)
        if game.turn_mode != TurnMode::Regular
            || !self.check_forgo
            || !game.options.is_enabled("enableStallingCheck")
        {
            return StepOutcome::next();
        }

        // Java: Arrays.stream(game.getActingTeam().getPlayers())
        //   .filter(pl -> UtilPlayer.hasBall(game, pl) && game.getFieldModel().getPlayerState(pl).isActive())
        //   .findFirst()
        //   .ifPresent(player -> { if (stallingExtension.isConsideredStalling(game, player)) { ... } });
        let active_team = game.active_team().clone();
        let stalling_player_id: Option<String> = active_team.players.iter().find(|pl| {
            UtilPlayer::has_ball(game, &pl.id)
                && game.field_model.player_state(&pl.id)
                    .map(|s| s.is_active())
                    .unwrap_or(false)
        }).map(|pl| pl.id.clone());

        if let Some(player_id) = stalling_player_id {
            if self.stalling_extension.is_considered_stalling(game, &player_id) {
                // Java: getResult().addReport(new ReportPlayerEvent(player.getId(), "is stalling"));
                // TODO: emit ReportPlayerEvent("is stalling") game event
                let turn_nr = game.turn_data().turn_nr;
                self.stalling_extension.handle_staller(game, &player_id, turn_nr, rng);
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
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn start_returns_next_default() {
        let mut game = make_game();
        let mut step = StepForgoneStalling::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_check_forgo_accepted() {
        let mut step = StepForgoneStalling::new();
        assert!(step.set_parameter(&StepParameter::CheckForgo(true)));
        assert!(step.check_forgo);
    }

    #[test]
    fn set_parameter_other_rejected() {
        let mut step = StepForgoneStalling::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(false)));
    }

    #[test]
    fn not_regular_turn_mode_skips() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Blitz;
        game.options.set("enableStallingCheck", "true");
        let mut step = StepForgoneStalling::new();
        step.check_forgo = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        // stalling not triggered — team result remains unchanged
        assert!(!game.game_result.home.stalled);
    }

    #[test]
    fn check_forgo_false_skips() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        game.options.set("enableStallingCheck", "true");
        let mut step = StepForgoneStalling::new();
        step.check_forgo = false;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn option_disabled_skips() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        // enableStallingCheck not set => disabled
        let mut step = StepForgoneStalling::new();
        step.check_forgo = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn no_ball_carrier_skips_without_panic() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        game.options.set("enableStallingCheck", "true");
        let mut step = StepForgoneStalling::new();
        step.check_forgo = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }
}
