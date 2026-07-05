/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.pass.StepIntercept`.
///
/// Step in the pass sequence to handle interceptions (BB2016).
/// - If no possible interceptors → goto failure label.
/// - Shows interception dialog; on interceptor choice → rolls interception.
/// - Re-roll (INTERCEPTION) and skill-auto-reroll (CATCH) supported.
/// - On success: publishes INTERCEPTOR_ID + next.
/// - On failure: publishes null INTERCEPTOR_ID + goto failure.
///
/// Init parameter: GOTO_LABEL_ON_FAILURE (mandatory).
/// Publishes: INTERCEPTOR_ID.
///
/// DEFERRED(Intercept-possibleInterceptors): UtilPassing.findInterceptors not yet ported.
/// DEFERRED(Intercept-roll): AgilityMechanic.minimumRollInterception + modifier factory not yet ported.
/// DEFERRED(Intercept-reroll): AbstractStepWithReRoll infrastructure not yet ported.
/// DEFERRED(Intercept-rangeRuler): UtilRangeRuler.createRangeRuler not yet ported.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepIntercept` (bb2016/pass).
pub struct StepIntercept {
    /// Java: `fGotoLabelOnFailure` — mandatory init param.
    goto_label_on_failure: String,
    /// Java: `fInterceptorId`
    interceptor_id: Option<String>,
    /// Java: `fInterceptorChosen`
    interceptor_chosen: bool,
}

impl StepIntercept {
    pub fn new() -> Self {
        Self {
            goto_label_on_failure: String::new(),
            interceptor_id: None,
            interceptor_chosen: false,
        }
    }

    fn execute_step(&self, _game: &mut Game) -> StepOutcome {
        // DEFERRED(Intercept-possibleInterceptors): find interceptors, show dialog, roll.
        // For now: no interceptors → goto failure immediately (safe default).
        StepOutcome::goto(&self.goto_label_on_failure)
            .publish(StepParameter::InterceptorId(None))
    }
}

impl Default for StepIntercept {
    fn default() -> Self { Self::new() }
}

impl Step for StepIntercept {
    fn id(&self) -> StepId { StepId::Intercept }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        match action {
            // Java: CLIENT_INTERCEPTOR_CHOICE — mapped to SelectPlayer in Rust Action enum
            Action::SelectPlayer { player_id } => {
                self.interceptor_id = Some(player_id.clone());
                self.interceptor_chosen = true;
                self.execute_step(game)
            }
            _ => self.execute_step(game),
        }
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
    fn id_is_intercept() {
        assert_eq!(StepIntercept::new().id(), StepId::Intercept);
    }

    #[test]
    fn no_interceptors_goto_failure() {
        let mut game = make_game();
        let mut step = StepIntercept::new();
        step.goto_label_on_failure = "fail".into();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::GotoLabel));
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
    }

    #[test]
    fn publishes_interceptor_id_null_on_failure() {
        let mut game = make_game();
        let mut step = StepIntercept::new();
        step.goto_label_on_failure = "fail".into();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::InterceptorId(None))));
    }

    #[test]
    fn set_parameter_goto_label() {
        let mut step = StepIntercept::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("x".into())));
        assert_eq!(step.goto_label_on_failure, "x");
    }

    #[test]
    fn set_parameter_interceptor_id_accepted() {
        let mut step = StepIntercept::new();
        assert!(step.set_parameter(&StepParameter::InterceptorId(Some("p1".into()))));
        assert_eq!(step.interceptor_id.as_deref(), Some("p1"));
    }

    #[test]
    fn set_parameter_interceptor_id_none_accepted() {
        let mut step = StepIntercept::new();
        assert!(step.set_parameter(&StepParameter::InterceptorId(None)));
        assert!(step.interceptor_id.is_none());
    }
}
