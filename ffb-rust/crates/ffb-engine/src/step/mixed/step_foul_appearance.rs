/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.StepFoulAppearance`.
///
/// Handles the Foul Appearance skill in the block/attack sequence.
/// Needs `GOTO_LABEL_ON_FAILURE` initialisation parameter.
///
/// Java: the entire roll logic lives in `executeStepHooks(this, state)` (not yet ported).
/// When hooks are ported, a failed Foul Appearance roll will produce a
/// `StepOutcome::goto(&state.goto_label_on_failure)`.
///
/// Java: `StepFoulAppearance extends AbstractStepWithReRoll` (mixed, BB2020 + BB2025).
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepFoulAppearance` (mixed, BB2020 + BB2025).
pub struct StepFoulAppearance {
    /// Java: `state.goToLabelOnFailure` (mandatory init param GOTO_LABEL_ON_FAILURE).
    pub goto_label_on_failure: String,
}

impl StepFoulAppearance {
    pub fn new(goto_label_on_failure: impl Into<String>) -> Self {
        Self { goto_label_on_failure: goto_label_on_failure.into() }
    }

    fn execute_step(&self, _game: &mut Game) -> StepOutcome {
        // Java: getGameState().executeStepHooks(this, state)
        // TODO(StepHooks port): roll Foul Appearance, offer re-roll dialog, on failure:
        //   return StepOutcome::goto(&self.goto_label_on_failure)
        StepOutcome::next()
    }
}

impl Default for StepFoulAppearance {
    fn default() -> Self { Self::new("") }
}

impl Step for StepFoulAppearance {
    fn id(&self) -> StepId { StepId::FoulAppearance }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnFailure(v) => { self.goto_label_on_failure = v.clone(); true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{test_team, StepAction};
    use ffb_model::enums::Rules;
    use ffb_model::model::game::Game;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn id_is_foul_appearance() {
        assert_eq!(StepFoulAppearance::new("fail").id(), StepId::FoulAppearance);
    }

    #[test]
    fn start_returns_next_step() {
        let mut step = StepFoulAppearance::new("fail");
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_goto_label_on_failure() {
        let mut step = StepFoulAppearance::new("old_label");
        let accepted = step.set_parameter(&StepParameter::GotoLabelOnFailure("new_label".into()));
        assert!(accepted);
        assert_eq!(step.goto_label_on_failure, "new_label");
    }

    #[test]
    fn set_parameter_rejects_unknown() {
        let mut step = StepFoulAppearance::new("fail");
        let rejected = !step.set_parameter(&StepParameter::EndTurn(true));
        assert!(rejected);
    }
}
