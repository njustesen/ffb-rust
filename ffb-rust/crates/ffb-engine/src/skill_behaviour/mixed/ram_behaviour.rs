use crate::skill_behaviour::SkillBehaviour;

/// Ram: may follow up a push and push again (multi-edition).
///
/// Armour/injury-modifier-only behaviour: registers an `AvOrInjModification` modifier.
/// No step hook is registered; `execute_step_hook` is not overridden.
///
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.mixed.RamBehaviour`.
pub struct RamBehaviour;

impl RamBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for RamBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for RamBehaviour {
    fn name(&self) -> &'static str { "RamBehaviour" }
    // Armour/injury-modifier-only behaviour — no step hook. execute_step_hook returns false (default).
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = RamBehaviour::new();
        assert_eq!(b.name(), "RamBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = RamBehaviour::default();
        assert_eq!(b.name(), "RamBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = RamBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2025,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = RamBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
}
