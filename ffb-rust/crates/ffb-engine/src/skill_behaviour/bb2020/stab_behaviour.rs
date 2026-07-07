use crate::skill_behaviour::SkillBehaviour;

/// BB2020 Stab skill behaviour. StepModifier on StepStab: if player has Stab, rolls stab injury
/// (no block roll), handles multiple-target stab. Mirrors Java
/// `com.fumbbl.ffb.server.skillbehaviour.bb2020.StabBehaviour`.
pub struct StabBehaviour;

impl StabBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for StabBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for StabBehaviour {
    fn name(&self) -> &'static str { "StabBehaviour" }

    /// Java `StepModifier<StepStab, StepState>.handleExecuteStepHook`: if player has Stab, rolls
    /// stab injury (no block roll), handles multiple-target stab. Returns false always.
    /// TODO(hook-infra): needs step stab state, injury results.
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
        let b = StabBehaviour::new();
        assert_eq!(b.name(), "StabBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = StabBehaviour::default();
        assert_eq!(b.name(), "StabBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = StabBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2020,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = StabBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
}
