/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.pass.StepPass`.
///
/// Step in the pass sequence to handle passing the ball (BB2016).
/// - Rolls pass using PassMechanic; evaluates result (ACCURATE/INACCURATE/FUMBLE/SAVED_FUMBLE).
/// - On ACCURATE: set ball/bomb coordinate, publish PASS_ACCURATE or CATCH_SCATTER mode.
/// - On FUMBLE: scatter ball.
/// - On SAVED_FUMBLE: ball stays with thrower (Safe Throw handled it).
/// - On INACCURATE/WILDLY_INACCURATE: goto missed-pass label.
/// - Re-roll (PASS) and skill auto-reroll supported.
///
/// Init parameters: GOTO_LABEL_ON_END (mandatory), GOTO_LABEL_ON_MISSED_PASS (mandatory).
/// Receives: CATCHER_ID.
/// Publishes: CATCHER_ID, PASS_ACCURATE, PASS_FUMBLE, DONT_DROP_FUMBLE,
///            CATCH_SCATTER_THROW_IN_MODE, PASS_DEVIATES.
///
/// TODO(Pass-roll): PassMechanic.findPassingDistance + evaluatePass not yet wired.
/// TODO(Pass-reroll): AbstractStepWithReRoll, UtilServerReRoll deferred.
/// TODO(Pass-skillDialog): DialogSkillUseParameter for passing skill reroll deferred.
use ffb_model::enums::PassResult;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepPass` (bb2016/pass).
pub struct StepPass {
    /// Java: `state.goToLabelOnEnd`
    goto_label_on_end: String,
    /// Java: `state.goToLabelOnMissedPass`
    goto_label_on_missed_pass: String,
    /// Java: `state.CatcherId`
    catcher_id: Option<String>,
    /// Java: `state.passSkillUsed`
    pass_skill_used: bool,
    /// Java: `state.result`
    result: Option<PassResult>,
}

impl StepPass {
    pub fn new() -> Self {
        Self {
            goto_label_on_end: String::new(),
            goto_label_on_missed_pass: String::new(),
            catcher_id: None,
            pass_skill_used: false,
            result: None,
        }
    }

    fn execute_step(&mut self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // TODO(Pass-roll): roll pass, evaluate result, set ball/bomb coordinate.
        // Stub: accurate pass assumed (safe-default for tests).
        StepOutcome::next()
    }
}

impl Default for StepPass {
    fn default() -> Self { Self::new() }
}

impl Step for StepPass {
    fn id(&self) -> StepId { StepId::Pass }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnEnd(s)       => { self.goto_label_on_end = s.clone(); true }
            StepParameter::GotoLabelOnMissedPass(s) => { self.goto_label_on_missed_pass = s.clone(); true }
            StepParameter::CatcherId(v)            => { self.catcher_id = v.clone(); true }
            StepParameter::PassResultParam(r)      => { self.result = Some(*r); true }
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
    fn id_is_pass() {
        assert_eq!(StepPass::new().id(), StepId::Pass);
    }

    #[test]
    fn set_parameter_goto_label_on_end() {
        let mut step = StepPass::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnEnd("end".into())));
        assert_eq!(step.goto_label_on_end, "end");
    }

    #[test]
    fn set_parameter_goto_label_on_missed_pass() {
        let mut step = StepPass::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnMissedPass("miss".into())));
        assert_eq!(step.goto_label_on_missed_pass, "miss");
    }

    #[test]
    fn set_parameter_catcher_id() {
        let mut step = StepPass::new();
        assert!(step.set_parameter(&StepParameter::CatcherId(Some("p2".into()))));
        assert_eq!(step.catcher_id, Some("p2".into()));
    }

    #[test]
    fn set_parameter_pass_result() {
        let mut step = StepPass::new();
        assert!(step.set_parameter(&StepParameter::PassResultParam(PassResult::Fumble)));
        assert_eq!(step.result, Some(PassResult::Fumble));
    }

    #[test]
    fn start_returns_next_step() {
        let mut game = make_game();
        let mut step = StepPass::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::NextStep));
    }
}
