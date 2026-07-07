use crate::skill_behaviour::SkillBehaviour;

/// BB2020 SideStep skill behaviour. StepModifier on StepPushback: if defender has SideStep,
/// changes pushback mode to allow player to choose destination square. Mirrors Java
/// `com.fumbbl.ffb.server.skillbehaviour.bb2020.SideStepBehaviour`.
pub struct SideStepBehaviour;

impl SideStepBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for SideStepBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for SideStepBehaviour {
    fn name(&self) -> &'static str { "SideStepBehaviour" }

    /// Java `StepModifier<StepPushback, StepState>.handleExecuteStepHook`: if defender has
    /// SideStep, changes pushback mode to allow player to choose destination square. Returns false
    /// always.
    /// TODO(hook-infra): needs state.defender, state.pushbackSquares,
    /// state.startingPushbackSquare.
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
        let b = SideStepBehaviour::new();
        assert_eq!(b.name(), "SideStepBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = SideStepBehaviour::default();
        assert_eq!(b.name(), "SideStepBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = SideStepBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2020,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = SideStepBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
#[test]    fn name_is_not_empty() {        assert!(!SideStepBehaviour::new().name().is_empty());    }    #[test]    fn execute_step_hook_false_with_bb2020() {        use ffb_model::enums::Rules;        use crate::step::framework::test_team;        let b = SideStepBehaviour::new();        let mut game = ffb_model::model::game::Game::new(            test_team("home", 0), test_team("away", 0), Rules::Bb2020,        );        assert!(!b.execute_step_hook(&mut game));    }
}
