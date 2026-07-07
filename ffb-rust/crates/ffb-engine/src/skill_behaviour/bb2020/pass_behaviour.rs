use crate::skill_behaviour::SkillBehaviour;

/// BB2020 Pass skill behaviour. Extends AbstractPassBehaviour; adds StepModifier on
/// StepHailMaryPass: rolls pass for LONG_BOMB distance, evaluates result, handles fumble reroll
/// dialog and modifying skill. Mirrors Java
/// `com.fumbbl.ffb.server.skillbehaviour.bb2020.PassBehaviour`.
pub struct PassBehaviour;

impl PassBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for PassBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for PassBehaviour {
    fn name(&self) -> &'static str { "PassBehaviour" }

    /// Java extends AbstractPassBehaviour; adds
    /// `StepModifier<StepHailMaryPass, StepState>.handleExecuteStepHook`: rolls pass for
    /// LONG_BOMB distance, evaluates result, handles fumble reroll dialog and modifying skill
    /// (canAddStrengthToPass). Returns false always.
    /// TODO(hook-infra): needs state.usingModifyingSkill, state.roll, state.result,
    /// state.minimumRoll, state.passSkillUsed, state.goToLabelOnFailure.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        // TODO(hook-infra): step-specific state access (StepState.xxx)
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_is_noop_returns_false() {
        // Without step infra the hook always returns false.
        let b = PassBehaviour::new();
        assert_eq!(b.name(), "PassBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = PassBehaviour::default();
        assert_eq!(b.name(), "PassBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = PassBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2020,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = PassBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
#[test]    fn name_is_not_empty() {        assert!(!PassBehaviour::new().name().is_empty());    }    #[test]    fn execute_step_hook_false_with_bb2020() {        use ffb_model::enums::Rules;        use crate::step::framework::test_team;        let b = PassBehaviour::new();        let mut game = ffb_model::model::game::Game::new(            test_team("home", 0), test_team("away", 0), Rules::Bb2020,        );        assert!(!b.execute_step_hook(&mut game));    }
}
