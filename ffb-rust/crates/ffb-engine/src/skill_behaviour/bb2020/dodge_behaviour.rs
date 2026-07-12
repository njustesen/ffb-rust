use crate::skill_behaviour::SkillBehaviour;

/// BB2020 Dodge skill behaviour. Mirrors Java
/// `com.fumbbl.ffb.server.skillbehaviour.bb2020.DodgeBehaviour`, which just calls
/// `super(1, false)` on `AbstractDodgingBehaviour` with no BB2020-specific override.
///
/// The real `StepModifierTrait` logic (dodge-choice default, `ReportSkillUse`) is
/// `AbstractDodgingStepModifier`, registered directly by
/// `registry.rs::build_bb2020` as `AbstractDodgingBehaviour::register_into(&mut reg,
/// SkillId::Dodge, 1, false)` — see `skill_behaviour/mixed/abstract_dodging_behaviour.rs`.
/// This type is an intentionally inert marker (matches the BB2016 `DodgeBehaviour`
/// precedent of not double-registering already-real logic).
pub struct DodgeBehaviour;

impl DodgeBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for DodgeBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for DodgeBehaviour {
    fn name(&self) -> &'static str { "DodgeBehaviour" }
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
