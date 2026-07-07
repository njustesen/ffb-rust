use crate::skill_behaviour::SkillBehaviour;

/// Really Stupid: player rolls each activation; passes only if adjacent to a non-Really-Stupid teammate.
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2016.ReallyStupidBehaviour`.
pub struct ReallyStupidBehaviour;

impl ReallyStupidBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for ReallyStupidBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for ReallyStupidBehaviour {
    fn name(&self) -> &'static str { "ReallyStupidBehaviour" }

    /// Java `StepModifier.handleExecuteStepHook` logic (StepConfusion context):
    ///
    /// Same pattern as `BoneHeadBehaviour`, but `goodConditions` is computed differently:
    ///
    /// 1. **Good conditions check**: find any adjacent friendly player that does NOT have the
    ///    Really Stupid skill (`goodConditions = adjacentNonReallyStupidTeammates.count() > 0`).
    /// 2. Compute target number via `minimumRollConfusion(goodConditions)`:
    ///    - With adjacent non-RS teammate: lower target number.
    ///    - Without: higher target number.
    /// 3. Only roll if `StepState.doRoll` is set (avoids re-triggering on loop-back).
    /// 4. On **fail**:
    ///    - Cancel the player's remaining action (set action to NONE / stunned state).
    ///    - GOTO `StepState.goToLabelOnFailure`.
    /// 5. On **success**: continue normally.
    ///
    /// All step-local state fields are unavailable in the current Rust signature:
    // TODO(hook-infra): step-specific state (StepState.doRoll)
    // TODO(hook-infra): step-specific state (StepState.goToLabelOnFailure)
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = ReallyStupidBehaviour::new();
        assert_eq!(b.name(), "ReallyStupidBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = ReallyStupidBehaviour::default();
        assert_eq!(b.name(), "ReallyStupidBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = ReallyStupidBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2016,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = ReallyStupidBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
#[test]    fn name_is_not_empty() {        assert!(!ReallyStupidBehaviour::new().name().is_empty());    }    #[test]    fn execute_step_hook_false_with_bb2020() {        use ffb_model::enums::Rules;        use crate::step::framework::test_team;        let b = ReallyStupidBehaviour::new();        let mut game = ffb_model::model::game::Game::new(            test_team("home", 0), test_team("away", 0), Rules::Bb2020,        );        assert!(!b.execute_step_hook(&mut game));    }
}
