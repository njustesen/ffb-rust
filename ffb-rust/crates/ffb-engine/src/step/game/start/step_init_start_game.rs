// TODO: full implementation. Stub placeholder for TRANSLATION_TRACKER.md.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepId, StepOutcome};

pub struct StepInitStartGame;

impl StepInitStartGame {
    pub fn new() -> Self { Self }
}

impl Default for StepInitStartGame {
    fn default() -> Self { Self::new() }
}

impl Step for StepInitStartGame {
    fn id(&self) -> StepId { StepId::InitStartGame }
    fn start(&mut self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome { StepOutcome::next() }
    fn handle_command(&mut self, _action: &Action, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome { StepOutcome::next() }
}
