
/// Juggernaut: removes Wrestle/Sidestep/Stand Firm from block results on a Blitz action
/// (multi-edition).
///
/// **This modifier is dead/unreachable code** (Phase AAH audit), same reason as
/// `bb2025::juggernaut_behaviour` — targets `StepId::Juggernaut`, which nothing dispatches. Java's
/// BB2016/BB2020 `JuggernautBehaviour.java` (byte-identical to the bb2025 copy modulo package/
/// imports) is ported directly into `step/action/block/step_juggernaut.rs`, one shared
/// edition-agnostic file used by all 3 rulesets (confirmed complete during Phase AAH's
/// investigation). Left registered rather than deleted, matching the Wrestle/Stab/DumpOff
/// precedent — though note this particular file has no `register_into`/`SkillRegistry` entry at
/// all (only referenced from the separate, unrelated `util_skill_behaviours.rs` informational
/// list), so there's nothing to unregister here either way.
///
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.mixed.JuggernautBehaviour`.
pub struct JuggernautBehaviour;

impl JuggernautBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for JuggernautBehaviour {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_creates_instance_same_as_new() {
        let _a = JuggernautBehaviour::new();
        let _b = JuggernautBehaviour::default();
    }
}
