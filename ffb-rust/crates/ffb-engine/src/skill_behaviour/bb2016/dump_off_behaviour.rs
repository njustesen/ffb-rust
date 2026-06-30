use crate::skill_behaviour::SkillBehaviour;

/// Dump Off: ball-carrier may attempt an emergency pass when targeted by a block.
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2016.DumpOffBehaviour`.
pub struct DumpOffBehaviour;

impl DumpOffBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for DumpOffBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for DumpOffBehaviour {
    fn name(&self) -> &'static str { "DumpOffBehaviour" }

    /// Java `StepDumpOff.handleExecuteStepHook` logic (condensed):
    ///
    /// Pre-conditions (all must hold, otherwise skip):
    ///   - The defending player holds the ball (`game.getFieldModel().getBallCarrierId()`).
    ///   - The defending player is not confused or hypnotised.
    ///   - A Dump Off has not already been used this turn.
    ///
    /// 1. Ask the defending coach via dialog: "Use Dump Off?" (YES/NO).
    /// 2. On YES:
    ///    a. Set the turn mode to DUMP_OFF (`game.setTurnMode(TurnMode.DUMP_OFF)`).
    ///    b. Push a full pass-action sequence onto the step stack for the defender
    ///       (using `SequenceGenerator` / `PassSequence`), so that an emergency
    ///       pass is resolved immediately before the blocking player's block.
    ///    c. Continue the outer step after the sub-sequence completes.
    /// 3. On NO → `setNextAction(NEXT_STEP)` (block proceeds normally).
    ///
    /// TODO(hook-infra): step-local dump-off dialog fields and stack-push mechanism.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = DumpOffBehaviour::new();
        assert_eq!(b.name(), "DumpOffBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = DumpOffBehaviour::default();
        assert_eq!(b.name(), "DumpOffBehaviour");
    }
}
