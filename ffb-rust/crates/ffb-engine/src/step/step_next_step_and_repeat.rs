use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepAction};
use crate::step::framework::{StepId, StepParameter};

/// 1:1 translation of com.fumbbl.ffb.server.step.StepNextStepAndRepeat.
///
/// Advances to the next step in the sequence and repeats the current step
/// (via NEXT_STEP_AND_REPEAT action). No init params.
pub struct StepNextStepAndRepeat;

impl StepNextStepAndRepeat {
    pub fn new() -> Self { Self }
}

impl Default for StepNextStepAndRepeat {
    fn default() -> Self { Self::new() }
}

impl Step for StepNextStepAndRepeat {
    fn id(&self) -> StepId { StepId::NextStepAndRepeat }

    fn start(&mut self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        StepOutcome {
            action: StepAction::NextStepAndRepeat,
            goto_label: None,
            published: vec![],
            pushes: vec![],
            events: vec![],
            prompt: None,
        }
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.start(game, rng)
    }

    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use ffb_model::enums::Rules;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    #[test]
    fn id_is_next_step_and_repeat() {
        assert_eq!(StepNextStepAndRepeat::new().id(), StepId::NextStepAndRepeat);
    }

    #[test]
    fn start_returns_next_step_and_repeat_action() {
        let out = StepNextStepAndRepeat::new().start(&mut make_game(), &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStepAndRepeat);
    }

    #[test]
    fn set_parameter_returns_false() {
        assert!(!StepNextStepAndRepeat::new().set_parameter(&StepParameter::EndTurn(true)));
    }
}
