/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.pass.StepSafeThrow`.
///
/// Step in the pass sequence to handle skill SAFE_THROW (BB2016).
/// - If thrower lacks canCancelInterceptions property, skip.
/// - If interceptor can cancel the skill, skip.
/// - Rolls (2+): on success, nullify interceptor → NEXT_STEP.
/// - On failure: set ball/bomb to interceptor coordinate → goto failure.
/// - Re-roll (SAFE_THROW) supported.
///
/// Init parameter: GOTO_LABEL_ON_FAILURE (mandatory).
/// Receives: INTERCEPTOR_ID.
/// Publishes: INTERCEPTOR_ID (null on success).
///
/// TODO(SafeThrow-roll): AgilityMechanic.minimumRollSafeThrow not yet ported.
/// TODO(SafeThrow-reroll): AbstractStepWithReRoll deferred.
/// TODO(SafeThrow-property): NamedProperties.canCancelInterceptions check deferred.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepSafeThrow` (bb2016/pass).
pub struct StepSafeThrow {
    /// Java: `fGotoLabelOnFailure` — mandatory init param.
    goto_label_on_failure: String,
    /// Java: `fInterceptorId`
    interceptor_id: Option<String>,
}

impl StepSafeThrow {
    pub fn new() -> Self {
        Self {
            goto_label_on_failure: String::new(),
            interceptor_id: None,
        }
    }

    fn execute_step(&self, _game: &mut Game) -> StepOutcome {
        // No interceptor → skip to next (nothing to cancel).
        if self.interceptor_id.is_none() {
            return StepOutcome::next();
        }
        // TODO(SafeThrow-property): check canCancelInterceptions; if not applicable, skip.
        // TODO(SafeThrow-roll): roll SAFE_THROW (2+); on success, null interceptor → next.
        //                       on failure, set ball to interceptor coordinate → goto failure.
        StepOutcome::next()
    }
}

impl Default for StepSafeThrow {
    fn default() -> Self { Self::new() }
}

impl Step for StepSafeThrow {
    fn id(&self) -> StepId { StepId::SafeThrow }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnFailure(s) => { self.goto_label_on_failure = s.clone(); true }
            StepParameter::InterceptorId(v)      => { self.interceptor_id = v.clone(); true }
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
    fn id_is_safe_throw() {
        assert_eq!(StepSafeThrow::new().id(), StepId::SafeThrow);
    }

    #[test]
    fn no_interceptor_returns_next() {
        let mut game = make_game();
        let mut step = StepSafeThrow::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::NextStep));
    }

    #[test]
    fn set_parameter_goto_label_on_failure() {
        let mut step = StepSafeThrow::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("fail".into())));
        assert_eq!(step.goto_label_on_failure, "fail");
    }

    #[test]
    fn set_parameter_interceptor_id() {
        let mut step = StepSafeThrow::new();
        assert!(step.set_parameter(&StepParameter::InterceptorId(Some("p3".into()))));
        assert_eq!(step.interceptor_id, Some("p3".into()));
    }

    #[test]
    fn set_parameter_interceptor_id_none() {
        let mut step = StepSafeThrow::new();
        step.interceptor_id = Some("p3".into());
        assert!(step.set_parameter(&StepParameter::InterceptorId(None)));
        assert!(step.interceptor_id.is_none());
    }
}
