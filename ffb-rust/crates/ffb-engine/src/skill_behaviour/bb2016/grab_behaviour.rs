use crate::skill_behaviour::SkillBehaviour;

/// Grab: after a block push the attacker may move the defender to any free adjacent square.
/// Priority 3 in the step-modifier ordering.
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2016.GrabBehaviour`.
pub struct GrabBehaviour;

impl GrabBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for GrabBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for GrabBehaviour {
    fn name(&self) -> &'static str { "GrabBehaviour" }

    /// Java `StepModifier.handleExecuteStepHook` logic (StepBlockPush context, priority 3):
    ///
    /// 1. Check conditions: attacker has Grab, defender does not have Stand Firm / Side Step,
    ///    and the push is not a chain-push or sideline push.
    /// 2. Collect all free pushback squares adjacent to the attacker.
    /// 3. If **all** pushback squares are free: auto-grab — set `StepState.pushbackMode = GRAB`
    ///    and update the set of valid pushback squares accordingly.
    /// 4. If only some squares are free: show a dialog asking the coach whether to use Grab;
    ///    on confirmation set `StepState.pushbackMode = GRAB` and restrict pushback squares.
    /// 5. Publish the updated pushback square list to the step state.
    ///
    /// All step-local state fields are unavailable in the current Rust signature:
    // TODO(hook-infra): step-specific state (StepState.pushbackMode)
    // TODO(hook-infra): step-specific state (StepState.pushbackSquares / new pushback squares)
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = GrabBehaviour::new();
        assert_eq!(b.name(), "GrabBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = GrabBehaviour::default();
        assert_eq!(b.name(), "GrabBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = GrabBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2016,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = GrabBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
#[test]    fn name_is_not_empty() {        assert!(!GrabBehaviour::new().name().is_empty());    }    #[test]    fn execute_step_hook_false_with_bb2020() {        use ffb_model::enums::Rules;        use crate::step::framework::test_team;        let b = GrabBehaviour::new();        let mut game = ffb_model::model::game::Game::new(            test_team("home", 0), test_team("away", 0), Rules::Bb2020,        );        assert!(!b.execute_step_hook(&mut game));    }
}
