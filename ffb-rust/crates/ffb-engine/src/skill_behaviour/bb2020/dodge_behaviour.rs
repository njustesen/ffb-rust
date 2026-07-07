use crate::skill_behaviour::SkillBehaviour;

/// BB2020 Dodge skill behaviour.
/// Extends AbstractDodgingBehaviour with +1 dodge modifier. execute_step_hook delegates to
/// AbstractDodgingBehaviour step modifier logic. Mirrors Java
/// `com.fumbbl.ffb.server.skillbehaviour.bb2020.DodgeBehaviour`.
pub struct DodgeBehaviour;

impl DodgeBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for DodgeBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for DodgeBehaviour {
    fn name(&self) -> &'static str { "DodgeBehaviour" }

    /// Extends AbstractDodgingBehaviour with +1 dodge modifier (BB2020).
    /// execute_step_hook delegates to AbstractDodgingBehaviour step modifier logic.
    ///
    /// TODO(hook-infra): actual modifier application happens in StepModifier registered by
    /// AbstractDodgingBehaviour.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        // TODO(hook-infra): step-specific state access (StepState.xxx) not yet
        // available — implement fully once the step-hook infrastructure is ported.
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_is_noop_returns_false() {
        // Without step infra the hook always returns false.
        let b = DodgeBehaviour::new();
        assert_eq!(b.name(), "DodgeBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = DodgeBehaviour::default();
        assert_eq!(b.name(), "DodgeBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = DodgeBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2020,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = DodgeBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
#[test]    fn name_is_not_empty() {        assert!(!DodgeBehaviour::new().name().is_empty());    }    #[test]    fn execute_step_hook_false_with_bb2020() {        use ffb_model::enums::Rules;        use crate::step::framework::test_team;        let b = DodgeBehaviour::new();        let mut game = ffb_model::model::game::Game::new(            test_team("home", 0), test_team("away", 0), Rules::Bb2020,        );        assert!(!b.execute_step_hook(&mut game));    }
}
