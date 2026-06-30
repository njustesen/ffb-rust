/// 1:1 translation of com.fumbbl.ffb.server.step.action.pass.StepAnimosity (COMMON).
///
/// Handles the Animosity skill check during a pass action.
///
/// Mandatory init param: GOTO_LABEL_ON_FAILURE.
/// Expected preceding param: CATCHER_ID.
///
/// Logic:
/// - If bomb turn → NEXT_STEP (skip)
/// - Check animosity_exists(thrower, catcher):
///   - If false → NEXT_STEP
///   - If true → roll d6; if ≥ 2 → NEXT_STEP; else → GOTO failure
///
/// Stub: animosity_exists currently always returns false (AnimosityValueEvaluator not yet translated).
/// Stub: reroll dialog omitted (random agent always declines).
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_mechanics::bb2025::skill_mechanic::SkillMechanic;
use ffb_mechanics::mechanics::minimum_roll_animosity;
use ffb_mechanics::mechanics::is_skill_roll_successful;
use ffb_mechanics::skill_mechanic::SkillMechanic as SkillMechanicTrait;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

pub struct StepAnimosity {
    /// Java: state.gotoLabelOnFailure — mandatory.
    pub goto_label_on_failure: String,
    /// Java: state.catcherId — set by preceding step parameter.
    pub catcher_id: Option<String>,
}

impl StepAnimosity {
    pub fn new(goto_label_on_failure: impl Into<String>) -> Self {
        Self {
            goto_label_on_failure: goto_label_on_failure.into(),
            catcher_id: None,
        }
    }
}

impl Step for StepAnimosity {
    fn id(&self) -> StepId { StepId::Animosity }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // AbstractStepWithReRoll.handleCommand processes reroll responses.
        // Random agent always declines → re-execute directly.
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnFailure(v) => { self.goto_label_on_failure = v.clone(); true }
            StepParameter::CatcherId(v) => { self.catcher_id = v.clone(); true }
            _ => false,
        }
    }
}

impl StepAnimosity {
    fn execute_step(&self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: if (game.getTurnMode().isBombTurn()) → NEXT_STEP
        if game.turn_mode.is_bomb_turn() {
            return StepOutcome::next();
        }

        let thrower = game.thrower().map(|p| p.clone());
        let catcher = self.catcher_id.as_deref()
            .and_then(|id| game.player(id).map(|p| p.clone()));

        let mechanic = SkillMechanic::new();

        let do_roll = match (&thrower, &catcher) {
            (Some(t), Some(c)) => mechanic.animosity_exists(t, c),
            _ => false,
        };

        if do_roll {
            let roll = rng.d6();
            let min_roll = minimum_roll_animosity();
            let successful = is_skill_roll_successful(roll, min_roll);
            if successful {
                return StepOutcome::next();
            } else {
                // Random agent: no reroll → sufferingAnimosity = true → GOTO failure
                return StepOutcome::goto(&self.goto_label_on_failure);
            }
        }

        // Java: doRoll = false → NEXT_STEP
        StepOutcome::next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::{Rules, TurnMode};

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.home_playing = true;
        game
    }

    #[test]
    fn bomb_turn_skips_animosity_check() {
        let mut game = make_game();
        game.turn_mode = TurnMode::BombHome;
        let mut step = StepAnimosity::new("fail");
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn no_animosity_skill_returns_next() {
        let mut game = make_game();
        // No thrower set, no catcher → animosity_exists = false
        let mut step = StepAnimosity::new("fail");
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn catcher_id_parameter_accepted() {
        let mut step = StepAnimosity::new("fail");
        step.set_parameter(&StepParameter::CatcherId(Some("c1".into())));
        assert_eq!(step.catcher_id.as_deref(), Some("c1"));
    }

    #[test]
    fn goto_label_on_failure_param_accepted() {
        let mut step = StepAnimosity::new("fail");
        step.set_parameter(&StepParameter::GotoLabelOnFailure("other".into()));
        assert_eq!(step.goto_label_on_failure, "other");
    }

    #[test]
    fn regular_turn_no_animosity_returns_next() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        let mut step = StepAnimosity::new("fail");
        step.catcher_id = Some("c2".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        // animosity_exists stub returns false → always NEXT_STEP
        assert_eq!(out.action, StepAction::NextStep);
    }
}
