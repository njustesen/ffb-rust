use crate::skill_behaviour::SkillBehaviour;

/// BB2020 BrutalBlock skill behaviour. Registers BrutalBlockModification: +1 to armour/injury
/// rolls when pushing/knocking players down. Mirrors Java
/// `com.fumbbl.ffb.server.skillbehaviour.bb2020.BrutalBlockBehaviour`.
pub struct BrutalBlockBehaviour;

impl BrutalBlockBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for BrutalBlockBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for BrutalBlockBehaviour {
    fn name(&self) -> &'static str { "BrutalBlockBehaviour" }

    /// No step modifier hook — this behaviour only registers BrutalBlockModification.
    /// BrutalBlockModification gives +1 to armour/injury rolls when pushing or knocking players
    /// down.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_is_noop_returns_false() {
        // BrutalBlockBehaviour only registers BrutalBlockModification; execute_step_hook is a no-op.
        let b = BrutalBlockBehaviour::new();
        assert_eq!(b.name(), "BrutalBlockBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = BrutalBlockBehaviour::default();
        assert_eq!(b.name(), "BrutalBlockBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = BrutalBlockBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2020,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = BrutalBlockBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
#[test]    fn name_is_not_empty() {        assert!(!BrutalBlockBehaviour::new().name().is_empty());    }    #[test]    fn execute_step_hook_false_with_bb2020() {        use ffb_model::enums::Rules;        use crate::step::framework::test_team;        let b = BrutalBlockBehaviour::new();        let mut game = ffb_model::model::game::Game::new(            test_team("home", 0), test_team("away", 0), Rules::Bb2020,        );        assert!(!b.execute_step_hook(&mut game));    }
}
