use crate::skill_behaviour::SkillBehaviour;

/// Watch Out: nearby teammates gain a bonus on armour rolls (multi-edition).
///
/// Extends `AbstractDodgingBehaviour` with `priority = 2` and `requireUnusedSkill = true`.
/// Delegates entirely to the abstract parent's step logic; no additional override.
///
/// The full step logic is documented on `AbstractDodgingBehaviour::execute_step_hook`.
/// This struct adds no new behaviour on top of that base.
///
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.mixed.WatchOutBehaviour`.
pub struct WatchOutBehaviour;

impl WatchOutBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for WatchOutBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for WatchOutBehaviour {
    fn name(&self) -> &'static str { "WatchOutBehaviour" }
    // Delegates to AbstractDodgingBehaviour step logic (priority=2, requireUnusedSkill=true).
    // No additional execute_step_hook override. Returns false (default).
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = WatchOutBehaviour::new();
        assert_eq!(b.name(), "WatchOutBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = WatchOutBehaviour::default();
        assert_eq!(b.name(), "WatchOutBehaviour");
    }
}
