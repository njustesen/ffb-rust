use crate::skill_behaviour::SkillBehaviour;

/// BB2020 GhostlyFlames skill behaviour. Registers GhostlyFlamesModification: modifies injury
/// results for this player's attacks. Mirrors Java
/// `com.fumbbl.ffb.server.skillbehaviour.bb2020.GhostlyFlamesBehaviour`.
pub struct GhostlyFlamesBehaviour;

impl GhostlyFlamesBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for GhostlyFlamesBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for GhostlyFlamesBehaviour {
    fn name(&self) -> &'static str { "GhostlyFlamesBehaviour" }

    /// No step modifier hook — this behaviour only registers GhostlyFlamesModification.
    /// GhostlyFlamesModification modifies injury results for this player's attacks.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_is_noop_returns_false() {
        // GhostlyFlamesBehaviour only registers GhostlyFlamesModification; execute_step_hook is a no-op.
        let b = GhostlyFlamesBehaviour::new();
        assert_eq!(b.name(), "GhostlyFlamesBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = GhostlyFlamesBehaviour::default();
        assert_eq!(b.name(), "GhostlyFlamesBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = GhostlyFlamesBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2020,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = GhostlyFlamesBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
}
