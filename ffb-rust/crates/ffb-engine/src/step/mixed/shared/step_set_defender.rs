/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.shared.StepSetDefender`.
///
/// Sets the game's defender ID from an incoming BLOCK_DEFENDER_ID or GAZE_VICTIM_ID parameter.
/// When `ignore_null_value` is true a null/None ID is silently dropped instead of clearing the
/// defender.  Expects BLOCK_DEFENDER_ID and IGNORE_NULL_VALUE as init parameters.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepSetDefender` (mixed/shared, BB2020 + BB2025).
pub struct StepSetDefender {
    /// Java: `defenderId`
    defender_id: Option<String>,
    /// Java: `ignoreNullValue`
    ignore_null_value: bool,
}

impl StepSetDefender {
    pub fn new() -> Self {
        Self { defender_id: None, ignore_null_value: false }
    }

    fn execute_step(&self, game: &mut Game) -> StepOutcome {
        if self.defender_id.is_some() || !self.ignore_null_value {
            game.defender_id = self.defender_id.clone();
        }
        StepOutcome::next()
    }
}

impl Default for StepSetDefender {
    fn default() -> Self { Self::new() }
}

impl Step for StepSetDefender {
    fn id(&self) -> StepId { StepId::SetDefender }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    /// Java: `setParameter` stores BLOCK_DEFENDER_ID / GAZE_VICTIM_ID but does NOT consume them
    /// (returns `super.setParameter()` which is `false`).  IGNORE_NULL_VALUE is init-only → consumed.
    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::BlockDefenderId(v) => { self.defender_id = Some(v.clone()); false }
            StepParameter::GazeVictimId(v)    => { self.defender_id = v.clone(); false }
            StepParameter::IgnoreNullValue(v) => { self.ignore_null_value = *v; true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use ffb_model::enums::Rules;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn sets_defender_from_block_defender_id() {
        let mut step = StepSetDefender::new();
        step.set_parameter(&StepParameter::BlockDefenderId("p01".into()));
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert_eq!(game.defender_id.as_deref(), Some("p01"));
    }

    #[test]
    fn sets_defender_from_gaze_victim_id() {
        let mut step = StepSetDefender::new();
        step.set_parameter(&StepParameter::GazeVictimId(Some("p02".into())));
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert_eq!(game.defender_id.as_deref(), Some("p02"));
    }

    #[test]
    fn clears_defender_when_gaze_victim_is_none_and_not_ignoring_null() {
        let mut step = StepSetDefender::new();
        step.set_parameter(&StepParameter::GazeVictimId(None));
        step.set_parameter(&StepParameter::IgnoreNullValue(false));
        let mut game = make_game();
        game.defender_id = Some("old".into());
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert!(game.defender_id.is_none());
    }

    #[test]
    fn preserves_defender_when_null_and_ignore_null_value_set() {
        let mut step = StepSetDefender::new();
        step.set_parameter(&StepParameter::GazeVictimId(None));
        step.set_parameter(&StepParameter::IgnoreNullValue(true));
        let mut game = make_game();
        game.defender_id = Some("old".into());
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        // defender_id.is_none() because defender_id on step is None and ignoreNullValue=true → skip
        assert_eq!(game.defender_id.as_deref(), Some("old"));
    }

    #[test]
    fn block_defender_id_not_consumed() {
        let mut step = StepSetDefender::new();
        let consumed = step.set_parameter(&StepParameter::BlockDefenderId("p03".into()));
        assert!(!consumed, "BlockDefenderId should propagate (not consumed)");
    }

    #[test]
    fn ignore_null_value_is_consumed() {
        let mut step = StepSetDefender::new();
        let consumed = step.set_parameter(&StepParameter::IgnoreNullValue(true));
        assert!(consumed, "IgnoreNullValue (init-only) should be consumed");
    }

    #[test]
    fn id_is_set_defender() {
        assert_eq!(StepSetDefender::new().id(), StepId::SetDefender);
    }
}
