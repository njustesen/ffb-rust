use crate::skill_behaviour::SkillBehaviour;

/// BB2020 Animosity skill behaviour.
/// StepModifier on StepAnimosity: checks if animosity exists between thrower and catcher, rolls
/// 3+, handles reroll, sets sufferingAnimosity. Mirrors Java
/// `com.fumbbl.ffb.server.skillbehaviour.bb2020.AnimosityBehaviour`.
pub struct AnimosityBehaviour;

impl AnimosityBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for AnimosityBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for AnimosityBehaviour {
    fn name(&self) -> &'static str { "AnimosityBehaviour" }

    /// Java `StepModifier<StepAnimosity, StepState>.handleExecuteStepHook`:
    /// checks if animosity exists between thrower and catcher, rolls 3+, handles reroll,
    /// sets sufferingAnimosity. Returns false always.
    ///
    /// TODO(hook-infra): needs state.catcherId, state.doRoll, state.gotoLabelOnFailure.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        // TODO(hook-infra): step-specific state access (StepState.catcherId,
        // StepState.doRoll, StepState.gotoLabelOnFailure)
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_is_noop_returns_false() {
        // Without step infra the hook always returns false.
        let b = AnimosityBehaviour::new();
        assert_eq!(b.name(), "AnimosityBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = AnimosityBehaviour::default();
        assert_eq!(b.name(), "AnimosityBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = AnimosityBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2020,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = AnimosityBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
}
