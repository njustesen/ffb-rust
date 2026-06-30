/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.ttm.StepThrowTeamMate`.
///
/// Step in TTM sequence to perform the actual throw. All logic is delegated
/// to `executeStepHooks()` — the step acts as a dispatch point for engine hooks
/// that push scatter/catch sequences depending on throw result.
///
/// Init param: GOTO_LABEL_ON_FAILURE (mandatory).
/// Consumed params: THROWN_PLAYER_ID, THROWN_PLAYER_STATE, THROWN_PLAYER_HAS_BALL.
///
/// TODO(ThrowTTM-hooks): executeStepHooks not yet ported — stub returns NEXT_STEP.
/// TODO(ThrowTTM-useSkill): handleSkillCommand (CLIENT_USE_SKILL) deferred.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::enums::PlayerState;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepThrowTeamMate` inner `StepState` — fields promoted to struct level.
pub struct StepThrowTeamMate {
    /// Java: `state.goToLabelOnFailure` — mandatory init param.
    goto_label_on_failure: String,
    /// Java: `state.thrownPlayerId`
    thrown_player_id: Option<String>,
    /// Java: `state.thrownPlayerState`
    thrown_player_state: Option<PlayerState>,
    /// Java: `state.thrownPlayerHasBall`
    thrown_player_has_ball: bool,
}

impl StepThrowTeamMate {
    pub fn new() -> Self {
        Self {
            goto_label_on_failure: String::new(),
            thrown_player_id: None,
            thrown_player_state: None,
            thrown_player_has_ball: false,
        }
    }

    fn execute_step(&self, _game: &mut Game) -> StepOutcome {
        // Java: getGameState().executeStepHooks(this, state);
        // TODO(ThrowTTM-hooks): step hooks (scatterPlayer push, throw roll) deferred.
        StepOutcome::next()
    }
}

impl Default for StepThrowTeamMate {
    fn default() -> Self { Self::new() }
}

impl Step for StepThrowTeamMate {
    fn id(&self) -> StepId { StepId::ThrowTeamMate }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // TODO(ThrowTTM-useSkill): CLIENT_USE_SKILL → handleSkillCommand deferred.
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnFailure(s) => { self.goto_label_on_failure = s.clone(); true }
            StepParameter::ThrownPlayerId(v)     => { self.thrown_player_id = v.clone(); true }
            StepParameter::ThrownPlayerState(v)  => { self.thrown_player_state = Some(*v); true }
            StepParameter::ThrownPlayerHasBall(v)=> { self.thrown_player_has_ball = *v; true }
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
    fn id_is_throw_team_mate() {
        assert_eq!(StepThrowTeamMate::new().id(), StepId::ThrowTeamMate);
    }

    #[test]
    fn start_returns_next_step() {
        let mut game = make_game();
        let mut step = StepThrowTeamMate::new();
        step.goto_label_on_failure = "fail".into();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::NextStep));
    }

    #[test]
    fn set_parameter_goto_label_on_failure() {
        let mut step = StepThrowTeamMate::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("fail".into())));
        assert_eq!(step.goto_label_on_failure, "fail");
    }

    #[test]
    fn set_parameter_thrown_player_id() {
        let mut step = StepThrowTeamMate::new();
        assert!(step.set_parameter(&StepParameter::ThrownPlayerId(Some("p1".into()))));
        assert_eq!(step.thrown_player_id.as_deref(), Some("p1"));
    }

    #[test]
    fn set_parameter_thrown_player_has_ball() {
        let mut step = StepThrowTeamMate::new();
        assert!(step.set_parameter(&StepParameter::ThrownPlayerHasBall(true)));
        assert!(step.thrown_player_has_ball);
    }
}
