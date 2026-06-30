/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.foul.StepEjectPlayer`.
///
/// Step in foul sequence to handle ejecting a spotted fouler (BB2016).
/// - Puts fouler in the box (UtilBox.putPlayerIntoBox).
/// - Checks wasted skills, updates player-state-dependent properties.
/// - Publishes END_TURN.
/// - If fouler had the ball: publishes CATCH_SCATTER_THROW_IN_MODE::SCATTER_BALL + NEXT_STEP.
/// - Otherwise: goto end label.
///
/// Init parameter: GOTO_LABEL_ON_END (mandatory).
/// Receives: FOULER_HAS_BALL, ARGUE_THE_CALL_SUCCESSFUL.
/// Publishes: END_TURN, CATCH_SCATTER_THROW_IN_MODE.
///
/// TODO(EjectPlayer-box): UtilBox.putPlayerIntoBox / refreshBoxes not yet ported.
/// TODO(EjectPlayer-wastedSkills): UtilServerGame.checkForWastedSkills deferred.
/// TODO(EjectPlayer-hooks): executeStepHooks deferred.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{CatchScatterThrowInMode, Step, StepOutcome, StepId, StepParameter};

/// Java: `StepEjectPlayer` (bb2016/foul).
pub struct StepEjectPlayer {
    /// Java: `state.gotoLabelOnEnd` — mandatory init param.
    goto_label_on_end: String,
    /// Java: `state.foulerHasBall`
    fouler_has_ball: Option<bool>,
    /// Java: `state.argueTheCallSuccessful`
    argue_the_call_successful: Option<bool>,
}

impl StepEjectPlayer {
    pub fn new() -> Self {
        Self {
            goto_label_on_end: String::new(),
            fouler_has_ball: None,
            argue_the_call_successful: None,
        }
    }

    fn execute_step(&self, _game: &mut Game) -> StepOutcome {
        // TODO(EjectPlayer-box): put acting player into box.
        // TODO(EjectPlayer-wastedSkills): check wasted skills.
        let has_ball = self.fouler_has_ball.unwrap_or(false);
        if has_ball {
            StepOutcome::next()
                .publish(StepParameter::EndTurn(true))
                .publish(StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ScatterBall))
        } else {
            StepOutcome::goto(&self.goto_label_on_end)
                .publish(StepParameter::EndTurn(true))
        }
    }
}

impl Default for StepEjectPlayer {
    fn default() -> Self { Self::new() }
}

impl Step for StepEjectPlayer {
    fn id(&self) -> StepId { StepId::EjectPlayer }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnEnd(s)         => { self.goto_label_on_end = s.clone(); true }
            StepParameter::FoulerHasBall(v)          => { self.fouler_has_ball = Some(*v); true }
            StepParameter::ArgueTheCallSuccessful(v)  => { self.argue_the_call_successful = Some(*v); true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::Rules;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    #[test]
    fn id_is_eject_player() {
        assert_eq!(StepEjectPlayer::new().id(), StepId::EjectPlayer);
    }

    #[test]
    fn no_ball_goto_label() {
        let mut game = make_game();
        let mut step = StepEjectPlayer::new();
        step.goto_label_on_end = "end".into();
        step.fouler_has_ball = Some(false);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::GotoLabel));
        assert_eq!(out.goto_label.as_deref(), Some("end"));
    }

    #[test]
    fn fouler_had_ball_scatter_ball() {
        let mut game = make_game();
        let mut step = StepEjectPlayer::new();
        step.goto_label_on_end = "end".into();
        step.fouler_has_ball = Some(true);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::NextStep));
        assert!(out.published.iter().any(|p| matches!(p,
            StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ScatterBall))));
    }

    #[test]
    fn publishes_end_turn_always() {
        let mut game = make_game();
        let mut step = StepEjectPlayer::new();
        step.goto_label_on_end = "end".into();
        step.fouler_has_ball = Some(false);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
    }

    #[test]
    fn set_parameter_fouler_has_ball() {
        let mut step = StepEjectPlayer::new();
        assert!(step.set_parameter(&StepParameter::FoulerHasBall(true)));
        assert_eq!(step.fouler_has_ball, Some(true));
    }

    #[test]
    fn set_parameter_argue_the_call() {
        let mut step = StepEjectPlayer::new();
        assert!(step.set_parameter(&StepParameter::ArgueTheCallSuccessful(true)));
        assert_eq!(step.argue_the_call_successful, Some(true));
    }
}
