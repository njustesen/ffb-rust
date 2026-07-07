use crate::skill_behaviour::SkillBehaviour;

/// Crushing Blow: +1 to armour rolls after a Knock Down result (multi-edition).
///
/// Injury-modifier-only behaviour: registers a `CrushingBlowModification` injury modifier.
/// No step hook is registered; `execute_step_hook` is not overridden.
///
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.mixed.CrushingBlowBehaviour`.
pub struct CrushingBlowBehaviour;

impl CrushingBlowBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for CrushingBlowBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for CrushingBlowBehaviour {
    fn name(&self) -> &'static str { "CrushingBlowBehaviour" }
    // Injury-modifier-only behaviour — no step hook. execute_step_hook returns false (default).
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = CrushingBlowBehaviour::new();
        assert_eq!(b.name(), "CrushingBlowBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = CrushingBlowBehaviour::default();
        assert_eq!(b.name(), "CrushingBlowBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = CrushingBlowBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2025,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = CrushingBlowBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
}
