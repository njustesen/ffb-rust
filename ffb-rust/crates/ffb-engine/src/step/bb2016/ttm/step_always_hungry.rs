/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.ttm.StepAlwaysHungry`.
///
/// Step in TTM sequence to handle skill ALWAYS_HUNGRY. Rolls 2+ for "always hungry"
/// (eating the thrown player); on failure rolls 2+ "escape". Both rolls re-rollable.
///
/// Init params: GOTO_LABEL_ON_FAILURE (mandatory), GOTO_LABEL_ON_SUCCESS (mandatory).
/// Consumed param: THROWN_PLAYER_ID.
///
/// TODO(AlwaysHungry-reroll): AbstractStepWithReRoll / UtilServerReRoll deferred.
/// TODO(AlwaysHungry-property): NamedProperties.mightEatPlayerToThrow / UtilCards deferred.
/// TODO(AlwaysHungry-rollSkill): DiceInterpreter.isAlwaysHungrySuccessful/isEscapeFromAlwaysHungrySuccessful deferred.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepAlwaysHungry` (bb2016/ttm).
pub struct StepAlwaysHungry {
    /// Java: `fGotoLabelOnFailure` — mandatory init param.
    goto_label_on_failure: String,
    /// Java: `fGotoLabelOnSuccess` — mandatory init param.
    goto_label_on_success: String,
    /// Java: `fThrownPlayerId`
    thrown_player_id: Option<String>,
}

impl StepAlwaysHungry {
    pub fn new() -> Self {
        Self {
            goto_label_on_failure: String::new(),
            goto_label_on_success: String::new(),
            thrown_player_id: None,
        }
    }

    fn execute_step(&self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // If no thrown player → nothing to do.
        if self.thrown_player_id.is_none() {
            return StepOutcome::next();
        }
        // TODO(AlwaysHungry-property): check NamedProperties.mightEatPlayerToThrow.
        // TODO(AlwaysHungry-rollSkill): roll always-hungry (2+); on failure roll escape (2+).
        // Stub: no always-hungry skill present → proceed.
        StepOutcome::next()
    }
}

impl Default for StepAlwaysHungry {
    fn default() -> Self { Self::new() }
}

impl Step for StepAlwaysHungry {
    fn id(&self) -> StepId { StepId::AlwaysHungry }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnFailure(s) => { self.goto_label_on_failure = s.clone(); true }
            StepParameter::GotoLabelOnSuccess(s) => { self.goto_label_on_success = s.clone(); true }
            StepParameter::ThrownPlayerId(v)     => { self.thrown_player_id = v.clone(); true }
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
    fn id_is_always_hungry() {
        assert_eq!(StepAlwaysHungry::new().id(), StepId::AlwaysHungry);
    }

    #[test]
    fn no_thrown_player_returns_next() {
        let mut game = make_game();
        let mut step = StepAlwaysHungry::new();
        step.goto_label_on_failure = "fail".into();
        step.goto_label_on_success = "success".into();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::NextStep));
    }

    #[test]
    fn set_parameter_thrown_player_id() {
        let mut step = StepAlwaysHungry::new();
        assert!(step.set_parameter(&StepParameter::ThrownPlayerId(Some("p1".into()))));
        assert_eq!(step.thrown_player_id.as_deref(), Some("p1"));
    }

    #[test]
    fn set_parameter_goto_labels() {
        let mut step = StepAlwaysHungry::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("fail".into())));
        assert!(step.set_parameter(&StepParameter::GotoLabelOnSuccess("ok".into())));
        assert_eq!(step.goto_label_on_failure, "fail");
        assert_eq!(step.goto_label_on_success, "ok");
    }

    #[test]
    fn with_thrown_player_but_no_skill_returns_next() {
        let mut game = make_game();
        let mut step = StepAlwaysHungry::new();
        step.goto_label_on_failure = "fail".into();
        step.goto_label_on_success = "success".into();
        step.thrown_player_id = Some("p1".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        // No always-hungry skill → stub returns NEXT_STEP.
        assert!(matches!(out.action, StepAction::NextStep));
    }
}
