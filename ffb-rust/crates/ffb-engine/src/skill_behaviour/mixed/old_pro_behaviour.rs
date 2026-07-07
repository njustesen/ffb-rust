use crate::skill_behaviour::SkillBehaviour;

/// Old Pro: may attempt to avoid being Stunned once per game (multi-edition).
///
/// Injury-modifier-only behaviour: registers an `OldProModification` injury modifier.
/// No step hook is registered; `execute_step_hook` is not overridden.
///
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.mixed.OldProBehaviour`.
pub struct OldProBehaviour;

impl OldProBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for OldProBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for OldProBehaviour {
    fn name(&self) -> &'static str { "OldProBehaviour" }
    // Injury-modifier-only behaviour — no step hook. execute_step_hook returns false (default).
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = OldProBehaviour::new();
        assert_eq!(b.name(), "OldProBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = OldProBehaviour::default();
        assert_eq!(b.name(), "OldProBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = OldProBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2025,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = OldProBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }

    #[test]
    fn name_is_not_empty() {
        assert!(!OldProBehaviour::new().name().is_empty());
    }

    #[test]
    fn execute_step_hook_false_with_bb2020() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = OldProBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2020,
        );
        assert!(!b.execute_step_hook(&mut game));
    }
}
