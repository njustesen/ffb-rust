use crate::skill_behaviour::SkillBehaviour;

/// Dodge: handles block-dodge pushback decisions and fall-through.
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2016.DodgeBehaviour`.
///
/// Player modifier: +1 on all dodge rolls (applied elsewhere via player-modifier hooks).
pub struct DodgeBehaviour;

impl DodgeBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for DodgeBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for DodgeBehaviour {
    fn name(&self) -> &'static str { "DodgeBehaviour" }

    /// Java `StepModifier.handleExecuteStepHook` logic (StepBlockPush / StepBlockRoll context):
    ///
    /// 1. Check if push is a chain-push — if so, skip (dodge does not apply).
    /// 2. Check if defender would be pushed to a sideline square — if so, skip.
    /// 3. Check if defender would be pushed into the attacker's half — if so, skip.
    /// 4. If none of the above apply, auto-use Dodge (usingDodge = true):
    ///    - On use: restore `StepState.oldDefenderState` for the defender.
    ///    - On decline: set defender to FALLING.
    /// 5. Publish pushback-init parameters (pushback square list, pushback mode) to the step.
    ///
    /// All step-local state fields are unavailable in the current Rust signature:
    // TODO(hook-infra): step-specific state (StepState.usingDodge)
    // TODO(hook-infra): step-specific state (StepState.oldDefenderState)
    // TODO(hook-infra): step-specific state (pushback square list / pushback mode parameters)
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = DodgeBehaviour::new();
        assert_eq!(b.name(), "DodgeBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = DodgeBehaviour::default();
        assert_eq!(b.name(), "DodgeBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = DodgeBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2016,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = DodgeBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
}
