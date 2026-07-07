use crate::skill_behaviour::SkillBehaviour;

/// Blood Lust: vampire must feed or risk losing control.
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2016.BloodLustBehaviour`.
pub struct BloodLustBehaviour;

impl BloodLustBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for BloodLustBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for BloodLustBehaviour {
    fn name(&self) -> &'static str { "BloodLustBehaviour" }

    /// Java `StepBloodLust.handleExecuteStepHook` logic (condensed):
    ///
    /// 1. Roll a confusion roll (`minimumRollBloodLust()`).
    /// 2. On success → `setNextAction(NEXT_STEP)` (no further effect this activation).
    /// 3. On failure → `state.sufferingBloodLust = true`.
    ///    a. If a reroll is available (step.reRolledAction != BLOOD_LUST and
    ///       step.reRollSource is usable):
    ///       → consume the reroll, re-enter this hook (NEXT_STEP with loop).
    ///    b. If no reroll available:
    ///       → `publishParameter(MOVE_STACK, null)` (clear the current move stack)
    ///       → `setNextAction(GOTO_LABEL, state.gotoLabelOnFailure)`
    ///
    /// TODO(hook-infra): step-specific state (StepState.doRoll, StepState.sufferingBloodLust,
    ///                   StepState.gotoLabelOnFailure, step.reRolledAction, step.reRollSource).
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = BloodLustBehaviour::new();
        assert_eq!(b.name(), "BloodLustBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = BloodLustBehaviour::default();
        assert_eq!(b.name(), "BloodLustBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = BloodLustBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2016,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = BloodLustBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
}
