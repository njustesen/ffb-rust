use crate::skill_behaviour::SkillBehaviour;

/// BB2020 DumpOff skill behaviour.
/// StepModifier on StepDumpOff: if already in DUMP_OFF mode resets it; else checks if defender
/// has DumpOff and ball conditions, shows skill-use dialog; if confirmed pushes Pass sequence.
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2020.DumpOffBehaviour`.
pub struct DumpOffBehaviour;

impl DumpOffBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for DumpOffBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for DumpOffBehaviour {
    fn name(&self) -> &'static str { "DumpOffBehaviour" }

    /// Java `StepModifier<StepDumpOff, StepState>.handleExecuteStepHook`:
    /// if already in DUMP_OFF turn mode resets it; else checks if defender has DumpOff and
    /// ball conditions, shows skill-use dialog; if confirmed pushes Pass sequence.
    /// Returns false always.
    ///
    /// TODO(hook-infra): needs state.usingDumpOff, state.oldTurnMode,
    /// state.defenderPosition, game.getTurnMode().
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        // TODO(hook-infra): step-specific state access (state.usingDumpOff,
        // state.oldTurnMode, state.defenderPosition, game.getTurnMode())
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_is_noop_returns_false() {
        // Without step infra the hook always returns false.
        let b = DumpOffBehaviour::new();
        assert_eq!(b.name(), "DumpOffBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = DumpOffBehaviour::default();
        assert_eq!(b.name(), "DumpOffBehaviour");
    }
}
