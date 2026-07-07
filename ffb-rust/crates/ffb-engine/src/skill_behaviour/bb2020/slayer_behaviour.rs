use crate::skill_behaviour::SkillBehaviour;

/// BB2020 Slayer skill behaviour. StepModifier on injury step: if attacker has Slayer and target
/// is a Big Guy, applies additional injury modifier. Mirrors Java
/// `com.fumbbl.ffb.server.skillbehaviour.bb2020.SlayerBehaviour`.
pub struct SlayerBehaviour;

impl SlayerBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for SlayerBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for SlayerBehaviour {
    fn name(&self) -> &'static str { "SlayerBehaviour" }

    /// Java `StepModifier` on injury step: if attacker has Slayer and target is a Big Guy, applies
    /// additional injury modifier. Returns false always.
    /// TODO(hook-infra): needs injury step state, target player type check.
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
        let b = SlayerBehaviour::new();
        assert_eq!(b.name(), "SlayerBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = SlayerBehaviour::default();
        assert_eq!(b.name(), "SlayerBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = SlayerBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2020,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = SlayerBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
}
