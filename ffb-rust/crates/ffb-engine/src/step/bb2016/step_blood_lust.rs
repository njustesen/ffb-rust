use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2016.StepBloodLust.
///
/// Step in the block sequence to handle Blood Lust (Vampire must feed).
/// Entirely hook-driven: all logic is in executeStepHooks(this, state).
///
/// Init: optional GOTO_LABEL_ON_FAILURE.
/// Sets: MOVE_STACK for all steps on the stack (via hooks).
/// Java: ActionStatus (hook output state for BloodLust)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BloodLustStatus {
    None,
    Success,
    Failure,
}

pub struct StepBloodLust {
    /// Java: state.status (ActionStatus — hook output)
    pub status: BloodLustStatus,
    /// Java: state.goToLabelOnFailure
    pub goto_label_on_failure: Option<String>,
}

impl StepBloodLust {
    pub fn new() -> Self {
        Self {
            status: BloodLustStatus::None,
            goto_label_on_failure: None,
        }
    }
}

impl Default for StepBloodLust {
    fn default() -> Self { Self::new() }
}

impl Step for StepBloodLust {
    fn id(&self) -> StepId { StepId::BloodLust }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: commandStatus = super.handleCommand(pReceivedCommand)
        // Java: if (commandStatus == EXECUTE_STEP) { executeStep() }
        // All meaningful commands are handled by AbstractStepWithReRoll's super.handleCommand.
        // TODO: re-roll commands via AbstractStepWithReRoll (not yet translated)
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnFailure(label) => {
                self.goto_label_on_failure = Some(label.clone());
                true
            }
            _ => false,
        }
    }
}

impl StepBloodLust {
    fn execute_step(&mut self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: getGameState().executeStepHooks(this, state)
        // The entire step logic is hook-driven. The hooks inspect state.goToLabelOnFailure,
        // run the blood lust roll, and set state.status to SUCCESS or FAILURE.
        // TODO: executeStepHooks — hook infrastructure not yet translated.
        // Stub: always advance to NEXT_STEP (hooks will eventually set the real outcome).
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
        Game::new(home, away, Rules::Bb2016)
    }

    #[test]
    fn start_returns_next_step() {
        let mut step = StepBloodLust::new();
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn goto_label_on_failure_parameter_accepted() {
        let mut step = StepBloodLust::new();
        let accepted = step.set_parameter(&StepParameter::GotoLabelOnFailure("label_end".to_string()));
        assert!(accepted);
        assert_eq!(step.goto_label_on_failure.as_deref(), Some("label_end"));
    }

    #[test]
    fn step_id_is_blood_lust() {
        let step = StepBloodLust::new();
        assert_eq!(step.id(), StepId::BloodLust);
    }

    #[test]
    fn unrecognised_parameter_returns_false() {
        let mut step = StepBloodLust::new();
        let accepted = step.set_parameter(&StepParameter::EndTurn(true));
        assert!(!accepted);
    }

    #[test]
    fn handle_command_returns_next_step() {
        let mut step = StepBloodLust::new();
        let mut game = make_game();
        let action = Action::EndTurn;
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }
}
