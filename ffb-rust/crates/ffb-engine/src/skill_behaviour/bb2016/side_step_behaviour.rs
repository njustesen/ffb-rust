use crate::skill_behaviour::SkillBehaviour;

/// Side Step: player chooses their own push square. Priority 2.
/// Asks defender if they want to use Side Step. On yes: sets pushbackMode=SIDE_STEP
/// and replaces pushback squares.
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2016.SideStepBehaviour`.
pub struct SideStepBehaviour;

impl SideStepBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for SideStepBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for SideStepBehaviour {
    fn name(&self) -> &'static str { "SideStepBehaviour" }

    /// Java logic (handleExecuteStepHook, priority 2):
    ///   1. Ask defender if they want to use Side Step (dialog).
    ///   2. On yes: set StepState.pushbackMode = SIDE_STEP.
    ///   3. Replace pushback squares with all unoccupied adjacent squares of the defender.
    ///   4. Reads/writes: StepState.usingStepBack, StepState.pushbackMode,
    ///      StepState.pushbackSquares.
    ///
    // TODO(hook-infra): step-specific state (StepState.usingStepBack)
    // TODO(hook-infra): step-specific state (StepState.pushbackMode)
    // TODO(hook-infra): step-specific state (StepState.pushbackSquares)
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = SideStepBehaviour::new();
        assert_eq!(b.name(), "SideStepBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = SideStepBehaviour::default();
        assert_eq!(b.name(), "SideStepBehaviour");
    }
}
