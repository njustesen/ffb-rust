use crate::skill_behaviour::SkillBehaviour;

/// Abstract base for dodge-modifying skill behaviours across editions.
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.mixed.AbstractDodgingBehaviour`.
pub struct AbstractDodgingBehaviour;

impl AbstractDodgingBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for AbstractDodgingBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for AbstractDodgingBehaviour {
    fn name(&self) -> &'static str { "AbstractDodgingBehaviour" }

    /// Java `AbstractDodgingBehaviour.handleExecuteStepHook` logic (StepDodge context):
    ///
    /// 1. Check if the defender has this skill; if `requireUnusedSkill` is set, also
    ///    verify the skill has not already been used (`isUsed()`).
    /// 2. If `StepState.usingDodge` is null, initialise it to
    ///    `StepState.oldDefenderState.hasTacklezones()`.
    /// 3. If `StepState.askForSkill` is true **and** `hasTacklezones` is true:
    ///    - Show the skill-use dialog to the active coach.
    ///    - Return `true` (waiting for a command).
    /// 4. Otherwise add a `ReportSkillUse` report entry.
    /// 5. Return `false` (no blocking wait).
    ///
    /// All step-local state fields are unavailable in the current Rust signature:
    // TODO(hook-infra): step-specific state (StepState.usingDodge)
    // TODO(hook-infra): step-specific state (StepState.oldDefenderState)
    // TODO(hook-infra): step-specific state (StepState.askForSkill)
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = AbstractDodgingBehaviour::new();
        assert_eq!(b.name(), "AbstractDodgingBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = AbstractDodgingBehaviour::default();
        assert_eq!(b.name(), "AbstractDodgingBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = AbstractDodgingBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2025,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = AbstractDodgingBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
}
