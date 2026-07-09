use crate::skill_behaviour::SkillBehaviour;

/// BB2020 SideStep skill behaviour.
///
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2020.SideStepBehaviour`.
///
/// **BB2020 vs BB2025 difference:**
///
/// 1. **Priority:** BB2020 registers the `StepPushback` modifier with priority **3**.
///    BB2025 registers it with priority **4**.
///
/// 2. **Skill class name:** BB2020 uses `SideStep` / `SideStepBehaviour`; BB2025 renames them to
///    `Sidestep` / `SidestepBehaviour` (lower-case 's' in "step"). The Rust side keeps
///    `SideStepBehaviour` for BB2020 to match the Java naming.
pub struct SideStepBehaviour;

impl SideStepBehaviour {
    pub fn new() -> Self { Self }

    /// Returns the priority used when registering the StepPushback modifier in BB2020.
    ///
    /// Java: `registerModifier(new StepModifier<StepPushback, StepState>(3) { ... })`
    pub const fn pushback_modifier_priority() -> u32 {
        3
    }
}

impl Default for SideStepBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for SideStepBehaviour {
    fn name(&self) -> &'static str { "SideStepBehaviour" }

    /// TODO(hook-infra): step-specific state access not yet wired.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// BB2020 registers the SideStep pushback modifier with priority 3.
    #[test]
    fn pushback_modifier_priority_is_3_in_bb2020() {
        assert_eq!(SideStepBehaviour::pushback_modifier_priority(), 3);
    }

    /// Priority is not 4 (BB2025 value).
    #[test]
    fn pushback_modifier_priority_is_not_4() {
        assert_ne!(SideStepBehaviour::pushback_modifier_priority(), 4);
    }

    #[test]
    fn name_is_correct() {
        assert_eq!(SideStepBehaviour::new().name(), "SideStepBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = SideStepBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2020,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = SideStepBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, before);
    }
}
