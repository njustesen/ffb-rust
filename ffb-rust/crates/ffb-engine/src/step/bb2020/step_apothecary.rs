use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// Applies apothecary to an injured player (BB2020).
/// TODO: full logic.
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2020.StepApothecary`.
pub struct StepApothecary;

impl StepApothecary {
    pub fn new() -> Self { Self }
}

impl Default for StepApothecary {
    fn default() -> Self { Self::new() }
}

impl Step for StepApothecary {
    fn id(&self) -> StepId { StepId::Apothecary }
    fn start(&mut self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome { StepOutcome::next() }
    fn handle_command(&mut self, _action: &Action, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome { StepOutcome::next() }
    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}
