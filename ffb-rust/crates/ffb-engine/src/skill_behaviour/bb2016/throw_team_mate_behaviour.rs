use crate::skill_behaviour::SkillBehaviour;

/// Throw Team-Mate: may throw a Small teammate instead of the ball.
/// Rolls throw (TTM mechanic minimumRoll). On success: pushes ScatterPlayer sequence.
/// On fumble: checks for unused Pass reroll source, else asks for team reroll.
/// Supports THROW_TEAM_MATE reroll.
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2016.ThrowTeamMateBehaviour`.
pub struct ThrowTeamMateBehaviour;

impl ThrowTeamMateBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for ThrowTeamMateBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for ThrowTeamMateBehaviour {
    fn name(&self) -> &'static str { "ThrowTeamMateBehaviour" }

    /// Java logic (handleExecuteStepHook):
    ///   1. Roll TTM throw: minimumRoll() per passing distance (TTM mechanic).
    ///   2. On success (roll >= minimumRoll):
    ///      a. Push ScatterPlayer sequence for the thrown teammate.
    ///   3. On fumble (roll of 1 before modifiers, or critical fail):
    ///      a. Check for an unused Pass skill reroll source (StepState.passReRollSource).
    ///      b. If available: use it automatically (no dialog).
    ///      c. Otherwise: ask team for a reroll via ReRolledActions.THROW_TEAM_MATE dialog.
    ///   4. On failure without reroll: scattered player lands at thrower's feet.
    ///   5. Support reroll via ReRolledActions.THROW_TEAM_MATE.
    ///   6. Reads/writes: StepState.throwRoll, StepState.minimumRoll,
    ///      StepState.passReRollSource, StepState.reRolledAction,
    ///      StepState.thrownPlayerId, StepState.targetCoordinate.
    ///
    // TODO(hook-infra): step-specific state (StepState.throwRoll)
    // TODO(hook-infra): step-specific state (StepState.minimumRoll)
    // TODO(hook-infra): step-specific state (StepState.passReRollSource)
    // TODO(hook-infra): step-specific state (StepState.reRolledAction)
    // TODO(hook-infra): step-specific state (StepState.thrownPlayerId)
    // TODO(hook-infra): step-specific state (StepState.targetCoordinate)
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = ThrowTeamMateBehaviour::new();
        assert_eq!(b.name(), "ThrowTeamMateBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = ThrowTeamMateBehaviour::default();
        assert_eq!(b.name(), "ThrowTeamMateBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = ThrowTeamMateBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2016,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = ThrowTeamMateBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
#[test]    fn name_is_not_empty() {        assert!(!ThrowTeamMateBehaviour::new().name().is_empty());    }    #[test]    fn execute_step_hook_false_with_bb2025() {        use ffb_model::enums::Rules;        use crate::step::framework::test_team;        let b = ThrowTeamMateBehaviour::new();        let mut game = ffb_model::model::game::Game::new(            test_team("home", 0), test_team("away", 0), Rules::Bb2025,        );        assert!(!b.execute_step_hook(&mut game));    }
}
