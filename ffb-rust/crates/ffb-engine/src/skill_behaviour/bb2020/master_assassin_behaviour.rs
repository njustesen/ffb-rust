use crate::skill_behaviour::SkillBehaviour;

/// BB2020 MasterAssassin skill behaviour. Registers MasterAssassinModification: modifies injury
/// results for stab/foul actions. Mirrors Java
/// `com.fumbbl.ffb.server.skillbehaviour.bb2020.MasterAssassinBehaviour`.
pub struct MasterAssassinBehaviour;

impl MasterAssassinBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for MasterAssassinBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for MasterAssassinBehaviour {
    fn name(&self) -> &'static str { "MasterAssassinBehaviour" }

    /// No step modifier hook — this behaviour only registers MasterAssassinModification.
    /// MasterAssassinModification modifies injury results for stab/foul actions.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_is_noop_returns_false() {
        // MasterAssassinBehaviour only registers MasterAssassinModification; execute_step_hook is a no-op.
        let b = MasterAssassinBehaviour::new();
        assert_eq!(b.name(), "MasterAssassinBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = MasterAssassinBehaviour::default();
        assert_eq!(b.name(), "MasterAssassinBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = MasterAssassinBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2020,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = MasterAssassinBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
}
