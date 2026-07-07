use crate::skill_behaviour::SkillBehaviour;

/// BB2020 Grab skill behaviour.
/// StepModifier on StepPushback: if attacker has Grab and conditions met (free square, no
/// conflicting skill, block action), shows dialog or sets pushbackMode=GRAB. Returns true when
/// consumed. Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2020.GrabBehaviour`.
pub struct GrabBehaviour;

impl GrabBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for GrabBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for GrabBehaviour {
    fn name(&self) -> &'static str { "GrabBehaviour" }

    /// Java `StepModifier<StepPushback, StepState>.handleExecuteStepHook`:
    /// checks if attacker has Grab, no conflicting skill, defender has free square, action is
    /// block type; if all pushback squares occupied shows dialog; changes pushbackMode to GRAB
    /// and removes non-selected squares. Returns true when grab is active, false otherwise.
    ///
    /// TODO(hook-infra): needs state.grabbing, state.freeSquareAroundDefender,
    /// state.startingPushbackSquare, state.pushbackSquares, state.pushbackMode,
    /// state.defender.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        // TODO(hook-infra): step-specific state access (state.grabbing,
        // state.freeSquareAroundDefender, state.startingPushbackSquare,
        // state.pushbackSquares, state.pushbackMode, state.defender)
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_is_noop_returns_false() {
        // Without step infra the hook always returns false.
        let b = GrabBehaviour::new();
        assert_eq!(b.name(), "GrabBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = GrabBehaviour::default();
        assert_eq!(b.name(), "GrabBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = GrabBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2020,
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
