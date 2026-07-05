/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.StepEndThrowKeg`.
///
/// Ends the throw-keg sequence by always pushing an `EndPlayerAction` sequence onto the
/// stack (endPlayerAction=true, endTurn from the `END_TURN` parameter).
///
/// Java: `StepEndThrowKeg` — extends `AbstractStep`.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepEndThrowKeg` (mixed, BB2020 + BB2025).
pub struct StepEndThrowKeg {
    /// Java: `endTurn`
    end_turn: bool,
}

impl StepEndThrowKeg {
    pub fn new() -> Self {
        Self { end_turn: false }
    }

    fn execute_step(&self, _game: &mut Game) -> StepOutcome {
        // Java: EndPlayerAction sequence is pushed with (endPlayerAction=true, endTurn).
        // In the Rust rewrite the driver owns the sequence stack; we publish the parameters
        // that StepEndPlayerAction / the driver need and move to the next step.
        StepOutcome::next()
            .publish(StepParameter::EndPlayerAction(true))
            .publish(StepParameter::EndTurn(self.end_turn))
    }
}

impl Default for StepEndThrowKeg {
    fn default() -> Self { Self::new() }
}

impl Step for StepEndThrowKeg {
    fn id(&self) -> StepId { StepId::EndThrowKeg }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::EndTurn(v) => { self.end_turn = *v; true }
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
    fn id_is_end_throw_keg() {
        assert_eq!(StepEndThrowKeg::new().id(), StepId::EndThrowKeg);
    }

    #[test]
    fn start_publishes_end_player_action_true() {
        let mut step = StepEndThrowKeg::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
        let has_epa = out.published.iter().any(|p| matches!(p, StepParameter::EndPlayerAction(true)));
        assert!(has_epa, "should publish EndPlayerAction(true)");
    }

    #[test]
    fn end_turn_false_by_default() {
        let mut step = StepEndThrowKeg::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        let has_end_turn_false = out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(false)));
        assert!(has_end_turn_false, "default end_turn should be false");
    }

    #[test]
    fn set_parameter_end_turn_updates_state() {
        let mut step = StepEndThrowKeg::new();
        let accepted = step.set_parameter(&StepParameter::EndTurn(true));
        assert!(accepted);
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        let has_end_turn_true = out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true)));
        assert!(has_end_turn_true, "should publish EndTurn(true) after set_parameter");
    }

    #[test]
    fn set_parameter_rejects_unknown() {
        let mut step = StepEndThrowKeg::new();
        assert!(!step.set_parameter(&StepParameter::EndPlayerAction(true)));
    }
}
