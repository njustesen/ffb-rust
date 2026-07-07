use crate::skill_behaviour::SkillBehaviour;

/// BB2020 CloudBurster skill behaviour.
/// Registers StepCloudBurster step (not a StepModifier): when a deflection occurs on a long pass,
/// forces the interceptor to re-roll. execute_step_hook is not applicable — the logic lives in
/// StepCloudBurster.executeStep(). Mirrors Java
/// `com.fumbbl.ffb.server.skillbehaviour.bb2020.CloudBursterBehaviour`.
pub struct CloudBursterBehaviour;

impl CloudBursterBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for CloudBursterBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for CloudBursterBehaviour {
    fn name(&self) -> &'static str { "CloudBursterBehaviour" }

    /// Not a StepModifier — CloudBursterBehaviour registers a full StepCloudBurster step.
    /// When deflection succeeds on a long pass, StepCloudBurster checks if thrower has
    /// canForceInterceptionRerollOfLongPasses, resets deflection, and re-pushes the intercept
    /// step.
    ///
    /// TODO(hook-infra): not applicable — logic lives in StepCloudBurster.executeStep().
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        // TODO(hook-infra): not applicable — logic lives in StepCloudBurster.executeStep()
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_is_noop_returns_false() {
        // Without step infra the hook always returns false.
        let b = CloudBursterBehaviour::new();
        assert_eq!(b.name(), "CloudBursterBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = CloudBursterBehaviour::default();
        assert_eq!(b.name(), "CloudBursterBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = CloudBursterBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2020,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = CloudBursterBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
}
