use crate::skill_behaviour::SkillBehaviour;

/// Abstract base for multi-block step modifiers across editions.
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.mixed.AbstractStepModifierMultipleBlock`.
pub struct AbstractStepModifierMultipleBlock;

impl AbstractStepModifierMultipleBlock {
    pub fn new() -> Self { Self }
}

impl Default for AbstractStepModifierMultipleBlock {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for AbstractStepModifierMultipleBlock {
    fn name(&self) -> &'static str { "AbstractStepModifierMultipleBlock" }

    /// Java `AbstractStepModifierMultipleBlock.handleExecuteStepHook` logic
    /// (StepBlockMultiple / StepDauntlessMultiple context):
    ///
    /// **First run** (`StepState.firstRun == true`):
    /// 1. Iterate `StepState.blockTargets`; for each target where `requiresRoll()` is true:
    ///    - Roll the relevant die.
    ///    - Record whether a re-roll is available for that target.
    /// 2. After all rolls, if any re-roll opportunity was collected:
    ///    - Show the re-roll choice dialog and return `true` (waiting for command).
    /// 3. Otherwise advance to `NEXT_STEP`.
    ///
    /// **Second run** (after dialog response):
    /// 4. If the coach chose to re-roll (`StepState.reRollTarget` is set):
    ///    - Apply the re-roll using `StepState.reRollSource`.
    ///    - Re-execute the roll for the chosen target.
    /// 5. Proceed to `NEXT_STEP`.
    ///
    /// All step-local state fields are unavailable in the current Rust signature:
    // TODO(hook-infra): step-specific state (StepState.firstRun)
    // TODO(hook-infra): step-specific state (StepState.blockTargets)
    // TODO(hook-infra): step-specific state (StepState.reRollTarget)
    // TODO(hook-infra): step-specific state (StepState.reRollSource)
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = AbstractStepModifierMultipleBlock::new();
        assert_eq!(b.name(), "AbstractStepModifierMultipleBlock");
    }

    #[test]
    fn default_has_correct_name() {
        let b = AbstractStepModifierMultipleBlock::default();
        assert_eq!(b.name(), "AbstractStepModifierMultipleBlock");
    }
}
