/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.pass.StepHailMaryPass`.
///
/// Step in the pass sequence to handle skill HAIL_MARY_PASS.
/// Delegates entirely to `executeStepHooks` in Java — the hooks handle the actual
/// roll logic injected from skill-behaviour classes.
///
/// Init parameter: GOTO_LABEL_ON_FAILURE (mandatory).
/// Sets stepParameter PASS_FUMBLE for all steps on the stack.
///
/// TODO(HailMaryPass-hooks): executeStepHooks infrastructure not yet ported — roll logic deferred.
use ffb_model::enums::PassResult;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepHailMaryPass` (bb2016/pass).
pub struct StepHailMaryPass {
    /// Java: `state.goToLabelOnFailure` — mandatory init param.
    goto_label_on_failure: String,
    /// Java: `state.result`
    result: Option<PassResult>,
    /// Java: `state.passSkillUsed`
    pass_skill_used: bool,
}

impl StepHailMaryPass {
    pub fn new() -> Self {
        Self {
            goto_label_on_failure: String::new(),
            result: None,
            pass_skill_used: false,
        }
    }

    fn execute_step(&self, _game: &mut Game) -> StepOutcome {
        // TODO(HailMaryPass-hooks): executeStepHooks not yet ported;
        // the hooks roll the pass, handle re-rolls, publish PASS_FUMBLE, and goto on failure.
        StepOutcome::next()
    }
}

impl Default for StepHailMaryPass {
    fn default() -> Self { Self::new() }
}

impl Step for StepHailMaryPass {
    fn id(&self) -> StepId { StepId::HailMaryPass }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnFailure(s) => { self.goto_label_on_failure = s.clone(); true }
            StepParameter::PassResultParam(r)    => { self.result = Some(*r); true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::{PassResult, Rules};

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    #[test]
    fn id_is_hail_mary_pass() {
        assert_eq!(StepHailMaryPass::new().id(), StepId::HailMaryPass);
    }

    #[test]
    fn set_parameter_goto_label_on_failure() {
        let mut step = StepHailMaryPass::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("fail".into())));
        assert_eq!(step.goto_label_on_failure, "fail");
    }

    #[test]
    fn set_parameter_pass_result() {
        let mut step = StepHailMaryPass::new();
        assert!(step.set_parameter(&StepParameter::PassResultParam(PassResult::Fumble)));
        assert_eq!(step.result, Some(PassResult::Fumble));
    }

    #[test]
    fn start_returns_next_step() {
        let mut game = make_game();
        let mut step = StepHailMaryPass::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::NextStep));
    }
}
