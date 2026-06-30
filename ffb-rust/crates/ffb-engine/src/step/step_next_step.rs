use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// TODO: full logic.
pub struct StepNextStep;

impl StepNextStep {
    pub fn new() -> Self { Self }
}

impl Default for StepNextStep {
    fn default() -> Self { Self::new() }
}

impl Step for StepNextStep {
    fn id(&self) -> StepId { StepId::NextStep }
    fn start(&mut self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome { StepOutcome::next() }
    fn handle_command(&mut self, _action: &Action, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome { StepOutcome::next() }
    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}
