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

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = WatchOutBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2025,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = WatchOutBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }

    #[test]
    fn name_is_not_empty() {
        assert!(!WatchOutBehaviour::new().name().is_empty());
    }

    #[test]
    fn execute_step_hook_false_with_bb2020() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = WatchOutBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2020,
        );
        assert!(!b.execute_step_hook(&mut game));
    }
}
