/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.foul.StepFoulChainsaw`.
///
/// Step in foul sequence to handle skill CHAINSAW (BB2016).
/// - If actor does NOT have blocksLikeChainsaw property → NEXT_STEP.
/// - Rolls chainsaw (1D6 vs minimum 2).
/// - On success: publish USING_CHAINSAW + NEXT_STEP.
/// - On failure: ask for re-roll (CHAINSAW) if available.
/// - If re-roll exhausted: apply InjuryTypeChainsaw to the attacker → goto failure.
///
/// Init parameter: GOTO_LABEL_ON_FAILURE (mandatory).
/// Publishes: USING_CHAINSAW, END_TURN, INJURY_RESULT.
///
/// TODO(FoulChainsaw-property): NamedProperties.blocksLikeChainsaw check deferred.
/// TODO(FoulChainsaw-roll): DiceInterpreter.minimumRollChainsaw / dice roller deferred.
/// TODO(FoulChainsaw-reroll): AbstractStepWithReRoll / UtilServerReRoll deferred.
/// TODO(FoulChainsaw-injury): InjuryTypeChainsaw / UtilServerInjury deferred.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepFoulChainsaw` (bb2016/foul).
pub struct StepFoulChainsaw {
    /// Java: `fGotoLabelOnFailure` — mandatory init param.
    goto_label_on_failure: String,
}

impl StepFoulChainsaw {
    pub fn new() -> Self {
        Self { goto_label_on_failure: String::new() }
    }

    fn execute_step(&self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // TODO(FoulChainsaw-property): check blocksLikeChainsaw.
        // TODO(FoulChainsaw-roll): roll chainsaw, handle re-roll, apply injury on failure.
        StepOutcome::next()
    }
}

impl Default for StepFoulChainsaw {
    fn default() -> Self { Self::new() }
}

impl Step for StepFoulChainsaw {
    fn id(&self) -> StepId { StepId::FoulChainsaw }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnFailure(s) => { self.goto_label_on_failure = s.clone(); true }
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
    fn id_is_foul_chainsaw() {
        assert_eq!(StepFoulChainsaw::new().id(), StepId::FoulChainsaw);
    }

    #[test]
    fn set_parameter_goto_label_on_failure() {
        let mut step = StepFoulChainsaw::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("fail".into())));
        assert_eq!(step.goto_label_on_failure, "fail");
    }

    #[test]
    fn start_returns_next_step() {
        let mut game = make_game();
        let mut step = StepFoulChainsaw::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::NextStep));
    }
}
