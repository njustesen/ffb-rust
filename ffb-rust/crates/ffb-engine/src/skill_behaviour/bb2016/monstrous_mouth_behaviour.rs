use crate::skill_behaviour::SkillBehaviour;

/// Monstrous Mouth: grants a catch re-roll (same pattern as CatchBehaviour re-roll injection).
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2016.MonstrousMouthBehaviour`.
pub struct MonstrousMouthBehaviour;

impl MonstrousMouthBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for MonstrousMouthBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for MonstrousMouthBehaviour {
    fn name(&self) -> &'static str { "MonstrousMouthBehaviour" }

    /// Java `StepModifier.handleExecuteStepHook` logic (StepCatch context):
    ///
    /// Follows the same pattern as `CatchBehaviour`:
    /// 1. Set `StepState.reRolledAction = PlayerAction::CATCH`.
    /// 2. Set `StepState.reRollSource` from the Monstrous Mouth skill entry.
    /// 3. Set `StepState.rerollCatch = true`.
    /// 4. Return `true` (hook consumed — no further modifiers process catch re-rolls).
    ///
    /// All step-local state fields are unavailable in the current Rust signature:
    // TODO(hook-infra): step-specific state (StepState.rerollCatch)
    // TODO(hook-infra): step reroll fields (StepState.reRolledAction, StepState.reRollSource)
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = MonstrousMouthBehaviour::new();
        assert_eq!(b.name(), "MonstrousMouthBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = MonstrousMouthBehaviour::default();
        assert_eq!(b.name(), "MonstrousMouthBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = MonstrousMouthBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2016,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = MonstrousMouthBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
}
