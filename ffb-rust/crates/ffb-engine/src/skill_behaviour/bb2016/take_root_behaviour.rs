use crate::skill_behaviour::SkillBehaviour;

/// Take Root: player may become rooted and unable to move.
/// Rolls confusion (minimumRollConfusion(true)) if not already rooted. On fail:
/// cancelPlayerAction(), WAITING_FOR_RE_ROLL or FAILURE status. Uses ReRolledActionFactory
/// to find reroll action.
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2016.TakeRootBehaviour`.
pub struct TakeRootBehaviour;

impl TakeRootBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for TakeRootBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for TakeRootBehaviour {
    fn name(&self) -> &'static str { "TakeRootBehaviour" }

    /// Java logic (handleExecuteStepHook):
    ///   1. If player is already rooted: skip (no-op).
    ///   2. Roll confusion check: minimumRollConfusion(true) (typically 2+ on D6).
    ///   3. On roll failure:
    ///      a. Call cancelPlayerAction() — cancels the current player action.
    ///      b. Check for reroll via ReRolledActionFactory.forSkill(TAKE_ROOT).
    ///      c. If reroll available: set StepState.status = WAITING_FOR_RE_ROLL.
    ///      d. If no reroll: set StepState.status = FAILURE; player becomes rooted.
    ///   4. On success: player proceeds normally (not rooted this activation).
    ///   5. Reads/writes: StepState.status, StepState.reRolledAction,
    ///      StepState.confusionRoll.
    ///
    // TODO(hook-infra): step-specific state (StepState.status)
    // TODO(hook-infra): step-specific state (StepState.reRolledAction)
    // TODO(hook-infra): step-specific state (StepState.confusionRoll)
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = TakeRootBehaviour::new();
        assert_eq!(b.name(), "TakeRootBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = TakeRootBehaviour::default();
        assert_eq!(b.name(), "TakeRootBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = TakeRootBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2016,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = TakeRootBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
}
