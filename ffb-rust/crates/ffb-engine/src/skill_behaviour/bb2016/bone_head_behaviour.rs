use crate::skill_behaviour::SkillBehaviour;

/// Bone Head: player may become confused and inactive.
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2016.BoneHeadBehaviour`.
pub struct BoneHeadBehaviour;

impl BoneHeadBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for BoneHeadBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for BoneHeadBehaviour {
    fn name(&self) -> &'static str { "BoneHeadBehaviour" }

    /// Java `StepBoneHead.handleExecuteStepHook` logic (condensed):
    ///
    /// 1. Roll a confusion roll (`minimumRollConfusion(true)`).
    /// 2. On success → `setNextAction(NEXT_STEP)`.
    /// 3. On failure:
    ///    a. Cancel the player's current action rights:
    ///       - `player.setBlitzUsed(true)`, `setPassUsed(true)`,
    ///         `setHandoverUsed(true)`, `setFoulUsed(true)`.
    ///    b. If the player was in the process of standing up:
    ///       → set player status to PRONE.
    ///    c. Otherwise:
    ///       → set player status to CONFUSED and INACTIVE.
    ///    d. If a reroll is available (BONE_HEAD reroll source):
    ///       → ask for reroll dialog; on retry go back to step 1.
    ///    e. If no reroll:
    ///       → publish END_PLAYER_ACTION.
    ///       → `setNextAction(GOTO_LABEL, state.gotoLabelOnFailure)`.
    ///
    /// TODO(hook-infra): step-specific state (StepState.doRoll,
    ///                   StepState.gotoLabelOnFailure).
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = BoneHeadBehaviour::new();
        assert_eq!(b.name(), "BoneHeadBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = BoneHeadBehaviour::default();
        assert_eq!(b.name(), "BoneHeadBehaviour");
    }
}
