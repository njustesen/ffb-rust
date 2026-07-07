use crate::skill_behaviour::SkillBehaviour;

/// BB2020 FoulAppearance skill behaviour.
/// Two StepModifiers: (1) on StepFoulAppearance — rolls 2+ for attacker before block, handles
/// reroll, on failure marks hasBlocked=true and goes to failure label. (2) on
/// StepFoulAppearanceMultiple — handles multi-block case with per-target rolls. Mirrors Java
/// `com.fumbbl.ffb.server.skillbehaviour.bb2020.FoulAppearanceBehaviour`.
pub struct FoulAppearanceBehaviour;

impl FoulAppearanceBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for FoulAppearanceBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for FoulAppearanceBehaviour {
    fn name(&self) -> &'static str { "FoulAppearanceBehaviour" }

    /// Java registers two StepModifiers:
    /// (1) `StepModifier<StepFoulAppearance>` — rolls 2+ for attacker before block, handles
    /// reroll, on failure marks hasBlocked=true and goes to failure label.
    /// (2) `StepModifier<StepFoulAppearanceMultiple>` — handles multi-block case with
    /// per-target rolls. Both return false.
    ///
    /// TODO(hook-infra): needs state.goToLabelOnFailure, step.getReRolledAction(),
    /// step.getReRollSource().
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        // TODO(hook-infra): step-specific state access (state.goToLabelOnFailure,
        // step.getReRolledAction(), step.getReRollSource())
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_is_noop_returns_false() {
        // Without step infra the hook always returns false.
        let b = FoulAppearanceBehaviour::new();
        assert_eq!(b.name(), "FoulAppearanceBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = FoulAppearanceBehaviour::default();
        assert_eq!(b.name(), "FoulAppearanceBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = FoulAppearanceBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2020,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = FoulAppearanceBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
}
