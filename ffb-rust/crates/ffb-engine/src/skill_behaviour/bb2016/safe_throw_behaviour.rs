use crate::skill_behaviour::SkillBehaviour;

/// Safe Throw: registers StepSafeThrow via registerStep — no execute_step_hook modifier.
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2016.SafeThrowBehaviour`.
pub struct SafeThrowBehaviour;

impl SafeThrowBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for SafeThrowBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for SafeThrowBehaviour {
    fn name(&self) -> &'static str { "SafeThrowBehaviour" }

    /// Java: only registers StepSafeThrow via `registerStep`; no `handleExecuteStepHook`
    /// override exists. Nothing to do here.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = SafeThrowBehaviour::new();
        assert_eq!(b.name(), "SafeThrowBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = SafeThrowBehaviour::default();
        assert_eq!(b.name(), "SafeThrowBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = SafeThrowBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2016,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = SafeThrowBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
}
