use crate::skill_behaviour::SkillBehaviour;

/// BB2020 AnimalSavagery skill behaviour.
/// StepModifier on StepAnimalSavagery: rolls confusion-style check; if failed, may injure random
/// teammate or show player-choice dialog. Mirrors Java
/// `com.fumbbl.ffb.server.skillbehaviour.bb2020.AnimalSavageryBehaviour`.
pub struct AnimalSavageryBehaviour;

impl AnimalSavageryBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for AnimalSavageryBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for AnimalSavageryBehaviour {
    fn name(&self) -> &'static str { "AnimalSavageryBehaviour" }

    /// Java `StepModifier<StepAnimalSavagery, StepState>.handleExecuteStepHook`:
    /// rolls confusion-style check; if failed, may injure random teammate or show
    /// player-choice dialog. Returns false always.
    ///
    /// TODO(hook-infra): needs state.status, state.goToLabelOnFailure,
    /// state.kickingPlayer etc.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        // TODO(hook-infra): step-specific state access (StepState.status,
        // StepState.goToLabelOnFailure, StepState.kickingPlayer)
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_is_noop_returns_false() {
        // Without step infra the hook always returns false.
        let b = AnimalSavageryBehaviour::new();
        assert_eq!(b.name(), "AnimalSavageryBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = AnimalSavageryBehaviour::default();
        assert_eq!(b.name(), "AnimalSavageryBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = AnimalSavageryBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2020,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = AnimalSavageryBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
#[test]    fn name_is_not_empty() {        assert!(!AnimalSavageryBehaviour::new().name().is_empty());    }    #[test]    fn execute_step_hook_false_with_bb2020() {        use ffb_model::enums::Rules;        use crate::step::framework::test_team;        let b = AnimalSavageryBehaviour::new();        let mut game = ffb_model::model::game::Game::new(            test_team("home", 0), test_team("away", 0), Rules::Bb2020,        );        assert!(!b.execute_step_hook(&mut game));    }
}
