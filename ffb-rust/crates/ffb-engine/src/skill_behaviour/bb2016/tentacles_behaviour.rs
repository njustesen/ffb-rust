use crate::skill_behaviour::SkillBehaviour;

/// Tentacles: may prevent a dodging or jumping player from escaping.
/// Asks opposing team to choose a Tentacles player. Rolls escape (2D6 vs ST values).
/// On fail: moves player back to coordinateFrom, publishes FEEDING_ALLOWED=false,
/// END_PLAYER_ACTION, goToLabelOnSuccess. Supports TENTACLES_ESCAPE reroll.
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2016.TentaclesBehaviour`.
pub struct TentaclesBehaviour;

impl TentaclesBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for TentaclesBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for TentaclesBehaviour {
    fn name(&self) -> &'static str { "TentaclesBehaviour" }

    /// Java logic (handleExecuteStepHook):
    ///   1. Only active when actingPlayer is dodging or jumping (check PlayerAction).
    ///   2. Show dialog: ask opposing team to choose a player with Tentacles to use.
    ///   3. Roll escape: 2D6 + actingPlayer.ST vs 2D6 + tentaclesPlayer.ST.
    ///   4. On escape failure:
    ///      a. Move actingPlayer back to StepState.coordinateFrom.
    ///      b. Publish FEEDING_ALLOWED = false report.
    ///      c. Publish END_PLAYER_ACTION.
    ///      d. Push GOTO_LABEL with StepState.goToLabelOnSuccess.
    ///   5. Support reroll via ReRolledActions.TENTACLES_ESCAPE.
    ///   6. Reads/writes: StepState.coordinateFrom, StepState.reRolledAction,
    ///      StepState.tentaclesPlayerId, StepState.goToLabelOnSuccess.
    ///
    // TODO(hook-infra): step-specific state (StepState.coordinateFrom)
    // TODO(hook-infra): step-specific state (StepState.reRolledAction)
    // TODO(hook-infra): step-specific state (StepState.tentaclesPlayerId)
    // TODO(hook-infra): step-specific state (StepState.goToLabelOnSuccess)
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = TentaclesBehaviour::new();
        assert_eq!(b.name(), "TentaclesBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = TentaclesBehaviour::default();
        assert_eq!(b.name(), "TentaclesBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = TentaclesBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2016,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = TentaclesBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
}
