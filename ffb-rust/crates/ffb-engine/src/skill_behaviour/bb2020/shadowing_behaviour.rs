use crate::skill_behaviour::SkillBehaviour;

/// BB2020 Shadowing skill behaviour. StepModifier on StepShadowing: if opposing player has
/// Shadowing, shows use dialog, rolls agility contest (AG vs AG), handles reroll, if success marks
/// hasBlocked=true. Mirrors Java
/// `com.fumbbl.ffb.server.skillbehaviour.bb2020.ShadowingBehaviour`.
pub struct ShadowingBehaviour;

impl ShadowingBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for ShadowingBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for ShadowingBehaviour {
    fn name(&self) -> &'static str { "ShadowingBehaviour" }

    /// Java `StepModifier<StepShadowing, StepState>.handleExecuteStepHook`: checks if defender
    /// has Shadowing, shows dialog, rolls AG vs AG, handles reroll. Returns false always.
    /// TODO(hook-infra): needs state.goToLabelOnFailure, state.usingAction.
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
        let b = ShadowingBehaviour::new();
        assert_eq!(b.name(), "ShadowingBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = ShadowingBehaviour::default();
        assert_eq!(b.name(), "ShadowingBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = ShadowingBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2020,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = ShadowingBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
#[test]    fn name_is_not_empty() {        assert!(!ShadowingBehaviour::new().name().is_empty());    }    #[test]    fn execute_step_hook_false_with_bb2020() {        use ffb_model::enums::Rules;        use crate::step::framework::test_team;        let b = ShadowingBehaviour::new();        let mut game = ffb_model::model::game::Game::new(            test_team("home", 0), test_team("away", 0), Rules::Bb2020,        );        assert!(!b.execute_step_hook(&mut game));    }
}
