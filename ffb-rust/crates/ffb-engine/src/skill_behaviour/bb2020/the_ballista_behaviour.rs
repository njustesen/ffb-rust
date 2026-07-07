use crate::skill_behaviour::SkillBehaviour;

/// BB2020 TheBallista skill behaviour. StepModifier for TheBallista: extends throw team mate range
/// by allowing long bomb distance. Mirrors Java
/// `com.fumbbl.ffb.server.skillbehaviour.bb2020.TheBallistaBehaviour`.
pub struct TheBallistaBehaviour;

impl TheBallistaBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for TheBallistaBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for TheBallistaBehaviour {
    fn name(&self) -> &'static str { "TheBallistaBehaviour" }

    /// Java `StepModifier` for TheBallista: on throw team mate, allows longer range throw (long
    /// bomb distance). Returns false always.
    /// TODO(hook-infra): needs throw team mate state.
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
        let b = TheBallistaBehaviour::new();
        assert_eq!(b.name(), "TheBallistaBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = TheBallistaBehaviour::default();
        assert_eq!(b.name(), "TheBallistaBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = TheBallistaBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2020,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = TheBallistaBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
}
