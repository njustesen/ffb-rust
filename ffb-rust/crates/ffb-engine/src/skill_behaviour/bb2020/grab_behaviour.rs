use crate::skill_behaviour::SkillBehaviour;

/// BB2020 Grab skill behaviour.
///
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2020.GrabBehaviour`.
///
/// **BB2020 vs BB2025 difference:**
///
/// The `StepModifier<StepPushback, StepState>` is registered with priority **4** in BB2020.
/// BB2025 registers it with priority **5**.
///
/// Priority determines evaluation order when multiple modifiers apply to the same step. A higher
/// number means lower priority (evaluated later). BB2025 deprioritises Grab relative to BB2020.
pub struct GrabBehaviour;

impl GrabBehaviour {
    pub fn new() -> Self { Self }

    /// Returns the priority used when registering the StepPushback modifier in BB2020.
    ///
    /// Java: `registerModifier(new StepModifier<StepPushback, StepState>(4) { ... })`
    pub const fn pushback_modifier_priority() -> u32 {
        4
    }
}

impl Default for GrabBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for GrabBehaviour {
    fn name(&self) -> &'static str { "GrabBehaviour" }

    /// TODO(hook-infra): step-specific state access not yet wired.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// BB2020 registers the Grab pushback modifier with priority 4.
    #[test]
    fn pushback_modifier_priority_is_4_in_bb2020() {
        assert_eq!(GrabBehaviour::pushback_modifier_priority(), 4);
    }

    /// Priority is not 5 (that is the BB2025 value).
    #[test]
    fn pushback_modifier_priority_is_not_5() {
        assert_ne!(GrabBehaviour::pushback_modifier_priority(), 5);
    }

    #[test]
    fn name_is_correct() {
        assert_eq!(GrabBehaviour::new().name(), "GrabBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = GrabBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2020,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = GrabBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, before);
    }
}
