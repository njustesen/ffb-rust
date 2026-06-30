use crate::skill_behaviour::SkillBehaviour;

/// Leap: player may jump over occupied squares during movement.
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2016.LeapBehaviour`.
pub struct LeapBehaviour;

impl LeapBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for LeapBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for LeapBehaviour {
    fn name(&self) -> &'static str { "LeapBehaviour" }

    /// Java `StepModifier.handleExecuteStepHook` logic (StepJump context):
    ///
    /// 1. Only fires when the player is performing a jump (leap) during movement.
    /// 2. Roll the agility check (target number derived from player agility, with +1 for Leap).
    /// 3. On **fail**:
    ///    - Publish `INJURY_TYPE = InjuryType::DROP_JUMP` to the step state.
    ///    - GOTO `StepState.goToLabelOnFailure` (triggers injury sequence for the jumping player).
    /// 4. On success: continue normally (player lands safely).
    /// 5. The step supports re-rolling via `ReRollSource::LEAP` if available.
    ///
    /// All step-local state fields are unavailable in the current Rust signature:
    // TODO(hook-infra): step-specific state (StepState.goToLabelOnFailure)
    // TODO(hook-infra): step-specific state (INJURY_TYPE publish target)
    // TODO(hook-infra): step reroll fields (ReRollSource::LEAP)
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = LeapBehaviour::new();
        assert_eq!(b.name(), "LeapBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = LeapBehaviour::default();
        assert_eq!(b.name(), "LeapBehaviour");
    }
}
