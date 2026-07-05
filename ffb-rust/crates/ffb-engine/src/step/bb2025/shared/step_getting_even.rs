use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.shared.StepGettingEven.
/// Opposing team may use their apothecary after a casualty (Getting Even apothecary rule).
/// headless: opposing apo dialog — client-only.
pub struct StepGettingEven {
    /// Java: playerId
    pub player_id: Option<String>,
    /// Java: keyword (Keyword enum) — stored as name until Keyword enum is ported
    pub keyword_name: Option<String>,
    // AbstractStepWithReRoll stubs
    pub re_rolled_action: Option<String>,
    pub re_roll_source: Option<String>,
}

impl StepGettingEven {
    pub fn new() -> Self {
        Self { player_id: None, keyword_name: None, re_rolled_action: None, re_roll_source: None }
    }
}

impl Default for StepGettingEven {
    fn default() -> Self { Self::new() }
}

impl Step for StepGettingEven {
    fn id(&self) -> StepId { StepId::GettingEven }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::PlayerId(v) => { self.player_id = Some(v.clone()); true }
            _ => false,
        }
    }
}

impl StepGettingEven {
    fn execute_step(&self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // client-only: offer opposing apothecary dialog — client-side
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
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn start_returns_next() {
        let mut game = make_game();
        let mut step = StepGettingEven::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_player_id_accepted() {
        let mut step = StepGettingEven::new();
        assert!(step.set_parameter(&StepParameter::PlayerId("p1".into())));
        assert_eq!(step.player_id.as_deref(), Some("p1"));
    }

    #[test]
    fn handle_command_returns_next() {
        let mut game = make_game();
        let mut step = StepGettingEven::new();
        let out = step.handle_command(&Action::EndTurn, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }
}
