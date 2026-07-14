
/// Indomitable: player may use this skill after a successful Dauntless roll (multi-edition),
/// doubling the target's effective strength for the block.
///
/// **This modifier is dead/unreachable code** (Phase AAH audit) — it has no `register_into` at
/// all and is never inserted into `SkillRegistry`. In Java, `IndomitableBehaviour` registers a
/// priority-3 `StepModifier<StepDauntless, StepState>` chained onto the *same* step as Dauntless's
/// own priority-2 modifier (single-block path only — the multi-block equivalent is
/// `StepDoubleStrength`/`step/mixed/multiblock/step_double_strength.rs`, a distinct mechanism).
/// This single-block chain (headless-simplified: auto-declines when undecided rather than waiting
/// on a live dialog) is now ported directly into
/// `step/action/block/step_dauntless.rs::resolve_indomitable`, matching the "direct-in-step"
/// pattern already established for Wrestle/Stab/DumpOff/Bombardier/Dauntless itself.
///
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.mixed.IndomitableBehaviour`.
pub struct IndomitableBehaviour;

impl IndomitableBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for IndomitableBehaviour {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_creates_instance_same_as_new() {
        let _a = IndomitableBehaviour::new();
        let _b = IndomitableBehaviour::default();
    }
}
