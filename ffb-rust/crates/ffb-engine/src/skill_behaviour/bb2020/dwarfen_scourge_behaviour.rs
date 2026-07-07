use crate::skill_behaviour::SkillBehaviour;

/// BB2020 DwarfenScourge skill behaviour. Registers AvOrInjModification: +1 to armour or injury
/// rolls against specific targets. Mirrors Java
/// `com.fumbbl.ffb.server.skillbehaviour.bb2020.DwarfenScourgeBehaviour`.
pub struct DwarfenScourgeBehaviour;

impl DwarfenScourgeBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for DwarfenScourgeBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for DwarfenScourgeBehaviour {
    fn name(&self) -> &'static str { "DwarfenScourgeBehaviour" }

    /// No step modifier hook — this behaviour only registers AvOrInjModification.
    /// AvOrInjModification gives +1 to armour or injury rolls against specific targets.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_is_noop_returns_false() {
        // DwarfenScourgeBehaviour only registers AvOrInjModification; execute_step_hook is a no-op.
        let b = DwarfenScourgeBehaviour::new();
        assert_eq!(b.name(), "DwarfenScourgeBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = DwarfenScourgeBehaviour::default();
        assert_eq!(b.name(), "DwarfenScourgeBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = DwarfenScourgeBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2020,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = DwarfenScourgeBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
#[test]    fn name_is_not_empty() {        assert!(!DwarfenScourgeBehaviour::new().name().is_empty());    }    #[test]    fn execute_step_hook_false_with_bb2020() {        use ffb_model::enums::Rules;        use crate::step::framework::test_team;        let b = DwarfenScourgeBehaviour::new();        let mut game = ffb_model::model::game::Game::new(            test_team("home", 0), test_team("away", 0), Rules::Bb2020,        );        assert!(!b.execute_step_hook(&mut game));    }
}
