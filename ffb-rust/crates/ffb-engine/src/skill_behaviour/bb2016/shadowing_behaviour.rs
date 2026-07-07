use crate::skill_behaviour::SkillBehaviour;

/// Shadowing: may attempt to follow a dodging player.
/// Finds adjacent opposing players with Shadowing, asks opponent to choose one.
/// Rolls 2D6 + MA vs 2D6 + MA (shadowing escape). On fail: moves shadower to coordinateFrom.
/// Supports SHADOWING_ESCAPE reroll.
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2016.ShadowingBehaviour`.
pub struct ShadowingBehaviour;

impl ShadowingBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for ShadowingBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for ShadowingBehaviour {
    fn name(&self) -> &'static str { "ShadowingBehaviour" }

    /// Java logic (handleExecuteStepHook):
    ///   1. Find all adjacent opposing players with Shadowing skill.
    ///   2. Show dialog: ask opposing team to choose one shadower.
    ///   3. Roll 2D6 + actingPlayer.MA vs 2D6 + shadower.MA.
    ///   4. On escape failure: move shadower to StepState.coordinateFrom; publish coordinate report.
    ///   5. Support reroll via ReRolledActions.SHADOWING_ESCAPE.
    ///   6. Reads/writes: StepState.coordinateFrom, StepState.reRolledAction,
    ///      StepState.shadowingPlayerId.
    ///
    // TODO(hook-infra): step-specific state (StepState.coordinateFrom)
    // TODO(hook-infra): step-specific state (StepState.reRolledAction)
    // TODO(hook-infra): step-specific state (StepState.shadowingPlayerId)
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = ShadowingBehaviour::new();
        assert_eq!(b.name(), "ShadowingBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = ShadowingBehaviour::default();
        assert_eq!(b.name(), "ShadowingBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = ShadowingBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2016,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = ShadowingBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
#[test]    fn name_is_not_empty() {        assert!(!ShadowingBehaviour::new().name().is_empty());    }    #[test]    fn execute_step_hook_false_with_bb2020() {        use ffb_model::enums::Rules;        use crate::step::framework::test_team;        let b = ShadowingBehaviour::new();        let mut game = ffb_model::model::game::Game::new(            test_team("home", 0), test_team("away", 0), Rules::Bb2020,        );        assert!(!b.execute_step_hook(&mut game));    }
}
