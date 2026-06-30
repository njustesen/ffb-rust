/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.foul.StepEjectPlayer`.
///
/// Removes the spotted fouler from the field (puts them in the box) and ends the turn.
/// If the fouler had the ball, also scatters it.
///
/// Java: `executeStepHooks` is deferred; `UtilBox.putPlayerIntoBox` / `refreshBoxes`
/// are stubs (UtilBox not yet ported).
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{CatchScatterThrowInMode, Step, StepOutcome, StepId, StepParameter};

/// Java: `StepEjectPlayer` (mixed/foul, BB2020 + BB2025).
pub struct StepEjectPlayer {
    /// Java: `state.gotoLabelOnEnd`
    goto_label_on_end: String,
    /// Java: `state.foulerHasBall`
    fouler_has_ball: Option<bool>,
    /// Java: `state.argueTheCallSuccessful`
    argue_the_call_successful: Option<bool>,
    /// Java: `state.officiousRef`
    officious_ref: bool,
}

impl StepEjectPlayer {
    pub fn new() -> Self {
        Self {
            goto_label_on_end: String::new(),
            fouler_has_ball: None,
            argue_the_call_successful: None,
            officious_ref: false,
        }
    }

    fn execute_step(&self, game: &mut Game) -> StepOutcome {
        // Java: UtilBox.putPlayerIntoBox(game, actingPlayer.getPlayer())
        // TODO: UtilBox not yet ported — player remains on field until UtilBox is implemented
        // Java: UtilBox.refreshBoxes(game) — also deferred
        // Java: UtilServerGame.updatePlayerStateDependentProperties(this) — deferred
        let _ = game;

        if self.fouler_has_ball == Some(true) {
            // Java: setNextAction(StepAction.NEXT_STEP)
            StepOutcome::next()
                .publish(StepParameter::EndTurn(true))
                .publish(StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ScatterBall))
        } else {
            // Java: setNextAction(StepAction.GOTO_LABEL, state.gotoLabelOnEnd)
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
            StepParameter::GotoLabelOnEnd(v)         => { self.goto_label_on_end = v.clone(); false }
            StepParameter::FoulerHasBall(v)           => { self.fouler_has_ball = Some(*v); true }
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
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn id_is_eject_player() {
        assert_eq!(StepEjectPlayer::new().id(), StepId::EjectPlayer);
    }

    #[test]
    fn always_publishes_end_turn() {
        let mut step = StepEjectPlayer::new();
        step.set_parameter(&StepParameter::GotoLabelOnEnd("end".into()));
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        let has_end_turn = outcome.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true)));
        assert!(has_end_turn);
    }

    #[test]
    fn without_ball_gotos_label() {
        let mut step = StepEjectPlayer::new();
        step.set_parameter(&StepParameter::GotoLabelOnEnd("end".into()));
        step.set_parameter(&StepParameter::FoulerHasBall(false));
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        assert!(matches!(outcome.action, StepAction::GotoLabel));
    }

    #[test]
    fn with_ball_next_step_and_scatter() {
        let mut step = StepEjectPlayer::new();
        step.set_parameter(&StepParameter::GotoLabelOnEnd("end".into()));
        step.set_parameter(&StepParameter::FoulerHasBall(true));
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        assert!(matches!(outcome.action, StepAction::NextStep));
        let has_scatter = outcome.published.iter().any(|p| {
            matches!(p, StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ScatterBall))
        });
        assert!(has_scatter);
    }

    #[test]
    fn fouler_has_ball_param_consumed() {
        let mut step = StepEjectPlayer::new();
        // set_parameter returns true → consumed
        let consumed = step.set_parameter(&StepParameter::FoulerHasBall(true));
        assert!(consumed);
    }

    #[test]
    fn goto_label_on_end_not_consumed() {
        let mut step = StepEjectPlayer::new();
        // Java: init() stores it but does NOT consume → return false
        let consumed = step.set_parameter(&StepParameter::GotoLabelOnEnd("end".into()));
        assert!(!consumed);
    }
}
