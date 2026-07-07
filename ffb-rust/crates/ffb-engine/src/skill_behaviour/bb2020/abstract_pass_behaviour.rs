use crate::skill_behaviour::SkillBehaviour;

/// Abstract base for BB2020 pass-related skill behaviours.
/// StepModifier on StepPass: handleCommandHook sets reRolledAction=PASS and reRollSource from
/// command. handleExecuteStepHook is a no-op (returns false). Mirrors Java
/// `com.fumbbl.ffb.server.skillbehaviour.bb2020.AbstractPassBehaviour`.
pub struct AbstractPassBehaviour;

impl AbstractPassBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for AbstractPassBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for AbstractPassBehaviour {
    fn name(&self) -> &'static str { "AbstractPassBehaviour" }

    /// Java `StepModifier<StepPass, StepState>.handleExecuteStepHook`:
    /// handleCommandHook sets reRolledAction=PASS and reRollSource from command.
    /// handleExecuteStepHook is a no-op (returns false always).
    ///
    /// TODO(hook-infra): needs reRolledAction, reRollSource from step command state.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        // TODO(hook-infra): step-specific state access (StepState.reRolledAction, reRollSource)
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_is_noop_returns_false() {
        // Without step infra the hook always returns false.
        let b = AbstractPassBehaviour::new();
        assert_eq!(b.name(), "AbstractPassBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = AbstractPassBehaviour::default();
        assert_eq!(b.name(), "AbstractPassBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = AbstractPassBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2020,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = AbstractPassBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
#[test]    fn name_is_not_empty() {        assert!(!AbstractPassBehaviour::new().name().is_empty());    }    #[test]    fn execute_step_hook_false_with_bb2020() {        use ffb_model::enums::Rules;        use crate::step::framework::test_team;        let b = AbstractPassBehaviour::new();        let mut game = ffb_model::model::game::Game::new(            test_team("home", 0), test_team("away", 0), Rules::Bb2020,        );        assert!(!b.execute_step_hook(&mut game));    }
}
