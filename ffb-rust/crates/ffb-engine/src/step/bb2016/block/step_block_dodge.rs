/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.block.StepBlockDodge`.
///
/// Step in block sequence to handle skill DODGE.
///
/// Expects stepParameter OLD_DEFENDER_STATE to be set by a preceding step.
///
/// The BB2016 Dodge step uses an executeStepHooks pattern. The state struct `StepState`
/// holds the tristate `usingDodge` and the `oldDefenderState` that hooks inspect.
/// Without the hooks infrastructure, this step delegates to a no-op implementation.
use ffb_model::enums::PlayerState;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepBlockDodge.StepState` — inner class holding hook-visible state.
#[derive(Debug, Clone, Default)]
pub struct StepState {
    /// Java: `usingDodge` (Boolean — tristate: null/true/false)
    pub using_dodge: Option<bool>,
    /// Java: `oldDefenderState`
    pub old_defender_state: Option<PlayerState>,
}

/// Java: `StepBlockDodge` (bb2016/block).
pub struct StepBlockDodge {
    /// Java: `state` — inner StepState
    pub state: StepState,
}

impl StepBlockDodge {
    pub fn new() -> Self {
        Self { state: StepState::default() }
    }

    fn execute_step(&mut self, _game: &mut Game) -> StepOutcome {
        // Java: getGameState().executeStepHooks(this, state)
        // The hooks infrastructure is not yet ported. BB2016 dodge skill handling
        // is delegated to step hooks that inspect state.usingDodge and state.oldDefenderState.
        // TODO(step-hooks): executeStepHooks not yet ported — this is a hook-driven step.
        // For now, advance immediately (no-op path).
        StepOutcome::next()
    }
}

impl Default for StepBlockDodge {
    fn default() -> Self { Self::new() }
}

impl Step for StepBlockDodge {
    fn id(&self) -> StepId { StepId::BlockDodge }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: if (commandStatus == UNHANDLED_COMMAND && id == CLIENT_USE_SKILL)
        //   commandStatus = handleSkillCommand((ClientCommandUseSkill) command, state)
        // Java: handleSkillCommand sets state.usingDodge from the skill-use command
        // TODO(skill-command): handleSkillCommand translation deferred with hooks infra
        match action {
            Action::UseSkill { use_skill, .. } => {
                // Java: state.usingDodge = useSkillCommand.isSkillUsed()
                self.state.using_dodge = Some(*use_skill);
            }
            _ => {}
        }
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            // Java: case OLD_DEFENDER_STATE: state.oldDefenderState = (PlayerState) parameter.getValue()
            StepParameter::OldDefenderState(s) => { self.state.old_defender_state = Some(*s); true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::{Rules, PS_PRONE, PS_STANDING};

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    #[test]
    fn id_is_block_dodge() {
        assert_eq!(StepBlockDodge::new().id(), StepId::BlockDodge);
    }

    #[test]
    fn start_returns_next_step() {
        // Java: executeStepHooks — no-op path advances
        let mut step = StepBlockDodge::new();
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_old_defender_state_accepted() {
        let mut step = StepBlockDodge::new();
        let state = PlayerState::new(PS_PRONE);
        assert!(step.set_parameter(&StepParameter::OldDefenderState(state)));
        assert_eq!(step.state.old_defender_state.unwrap().base(), PS_PRONE);
    }

    #[test]
    fn use_skill_command_sets_using_dodge() {
        use ffb_mechanics::skills::SkillId;
        let mut step = StepBlockDodge::new();
        let mut game = make_game();
        step.handle_command(
            &Action::UseSkill { skill_id: SkillId::Dodge, use_skill: true },
            &mut game,
            &mut GameRng::new(0),
        );
        assert_eq!(step.state.using_dodge, Some(true));
    }

    #[test]
    fn unrecognised_parameter_returns_false() {
        let mut step = StepBlockDodge::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(false)));
    }

    #[test]
    fn old_defender_state_standing_accepted() {
        let mut step = StepBlockDodge::new();
        let state = PlayerState::new(PS_STANDING);
        step.set_parameter(&StepParameter::OldDefenderState(state));
        assert_eq!(step.state.old_defender_state.unwrap().base(), PS_STANDING);
    }
}
