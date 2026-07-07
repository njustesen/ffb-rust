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

    /// Java `StepDivingTackle.handleExecuteStepHook` logic (condensed):
    ///
    /// 1. Compute the dodge roll that the dodging player must equal or beat,
    ///    applying the −2 Diving Tackle modifier.
    /// 2. If the dodge would still succeed even with the modifier → skip (no benefit).
    /// 3. If the dodge would fail:
    ///    a. Check for BreakTackle interaction: if the dodger has BreakTackle and
    ///       their Strength offsets the DT penalty, skip.
    ///    b. Ask the opposing team's coach to select a Diving Tackle player from
    ///       the set of eligible tacklers adjacent to the target square.
    ///    c. On confirmation:
    ///       → publish `USING_DIVING_TACKLE`.
    ///       → place the chosen defender in the target square (they are laid prone).
    ///       → `setNextAction(GOTO_LABEL, state.gotoLabelOnSuccess)`.
    ///
    /// TODO(hook-infra): step-specific state (StepState diving-tackle player-id,
    ///                   StepState.gotoLabelOnSuccess, dodge modifier context).
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
