use crate::skill_behaviour::SkillBehaviour;

/// Jump Up (BB2016): player may stand up without spending movement, but must pass an agility roll.
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2016.JumpUpBehaviour`.
pub struct JumpUpBehaviour;

impl JumpUpBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for JumpUpBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for JumpUpBehaviour {
    fn name(&self) -> &'static str { "JumpUpBehaviour" }

    /// Java `StepModifier.handleExecuteStepHook` logic (StepStandUp context):
    ///
    /// 1. Only fires when the player is standing up and has not yet moved.
    /// 2. Only applies for `PlayerAction::BLOCK` or `PlayerAction::MULTIPLE_BLOCK`.
    /// 3. Roll the agility jump-up check (target number derived from player agility).
    /// 4. On **fail**:
    ///    - Set the player to prone.
    ///    - Set next action to `END_PLAYER_ACTION`.
    ///    - GOTO `StepState.goToLabelOnFailure`.
    /// 5. On success: continue normally (player stands up for free).
    /// 6. The step supports re-rolling via `ReRollSource::JUMP_UP` if available.
    ///
    /// All step-local state fields are unavailable in the current Rust signature:
    // TODO(hook-infra): step-specific state (StepState.goToLabelOnFailure)
    // TODO(hook-infra): step reroll fields (ReRollSource::JUMP_UP)
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = JumpUpBehaviour::new();
        assert_eq!(b.name(), "JumpUpBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = JumpUpBehaviour::default();
        assert_eq!(b.name(), "JumpUpBehaviour");
    }
}
