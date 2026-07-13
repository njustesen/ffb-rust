use crate::skill_behaviour::SkillBehaviour;

/// Diving Tackle: defender may be placed adjacent to the dodging player on a failed dodge.
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2016.DivingTackleBehaviour`.
pub struct DivingTackleBehaviour;

impl DivingTackleBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for DivingTackleBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for DivingTackleBehaviour {
    fn name(&self) -> &'static str { "DivingTackleBehaviour" }

    /// Dead stub (Phase AAJ): Diving Tackle's real logic — dodge-modifier recomputation
    /// (with/without Break Tackle), eligible-tackler lookup, and the coach-choice dialog
    /// round-trip — is ported directly into `step/action/move_/step_diving_tackle.rs`
    /// (`execute_step_bb2016`), matching the established Wrestle/Stab/DumpOff/Dauntless
    /// direct-in-step convention. This `skill_behaviour/` hook is never reached (not wired
    /// through `dispatch::execute_step_hooks`) and stays a harmless registered no-op.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = DivingTackleBehaviour::new();
        assert_eq!(b.name(), "DivingTackleBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = DivingTackleBehaviour::default();
        assert_eq!(b.name(), "DivingTackleBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = DivingTackleBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2016,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = DivingTackleBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
#[test]    fn name_is_not_empty() {        assert!(!DivingTackleBehaviour::new().name().is_empty());    }    #[test]    fn execute_step_hook_false_with_bb2020() {        use ffb_model::enums::Rules;        use crate::step::framework::test_team;        let b = DivingTackleBehaviour::new();        let mut game = ffb_model::model::game::Game::new(            test_team("home", 0), test_team("away", 0), Rules::Bb2020,        );        assert!(!b.execute_step_hook(&mut game));    }
}
