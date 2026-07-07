use crate::skill_behaviour::SkillBehaviour;

/// Animosity: player may refuse to pass/hand off to certain teammates.
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2016.AnimosityBehaviour`.
pub struct AnimosityBehaviour;

impl AnimosityBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for AnimosityBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for AnimosityBehaviour {
    fn name(&self) -> &'static str { "AnimosityBehaviour" }

    /// Java `StepAnimosity.handleExecuteStepHook` logic (condensed):
    ///
    /// Branch A — player is already suffering animosity (state.sufferingAnimosity == true):
    ///   1. Look up the catcher player by `state.catcherId`.
    ///   2. If catcher's race matches the player's accepted race OR action is not PASS/HAND_OFF:
    ///      → `setNextAction(NEXT_STEP)`  (animosity check passes, continue normally)
    ///   3. Otherwise:
    ///      → `setNextAction(GOTO_LABEL, state.gotoLabelOnFailure)`  (abort pass/hand-off)
    ///
    /// Branch B — first entry (state.sufferingAnimosity == false):
    ///   1. Roll `minimumRollAnimosity()` against the skill's confusion roll threshold.
    ///   2. On success → `setNextAction(NEXT_STEP)`.
    ///   3. On failure → `state.sufferingAnimosity = true`.
    ///   4. If a reroll is available for ANIMOSITY:
    ///      → ask for reroll dialog (ANIMOSITY reroll source).
    ///   5. If no reroll → `setNextAction(GOTO_LABEL, state.gotoLabelOnFailure)`.
    ///
    /// TODO(hook-infra): step-specific state (StepState.catcherId, StepState.doRoll,
    ///                   StepState.sufferingAnimosity, StepState.gotoLabelOnFailure).
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = AnimosityBehaviour::new();
        assert_eq!(b.name(), "AnimosityBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = AnimosityBehaviour::default();
        assert_eq!(b.name(), "AnimosityBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = AnimosityBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2016,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = AnimosityBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
#[test]    fn name_is_not_empty() {        assert!(!AnimosityBehaviour::new().name().is_empty());    }    #[test]    fn execute_step_hook_false_with_bb2020() {        use ffb_model::enums::Rules;        use crate::step::framework::test_team;        let b = AnimosityBehaviour::new();        let mut game = ffb_model::model::game::Game::new(            test_team("home", 0), test_team("away", 0), Rules::Bb2020,        );        assert!(!b.execute_step_hook(&mut game));    }
}
