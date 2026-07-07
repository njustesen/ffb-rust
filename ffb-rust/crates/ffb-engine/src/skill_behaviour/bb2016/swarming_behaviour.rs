use crate::skill_behaviour::SkillBehaviour;

/// Swarming: extra linemen may replace injured players at kickoff.
/// Two phases: initial (find reserve swarming players, roll dice for allowed count, show dialog,
/// set SWARMING turn mode) and endTurn (validate placed count, reset prone reserves to RESERVE,
/// restore KICKOFF turn mode).
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2016.SwarmingBehaviour`.
pub struct SwarmingBehaviour;

impl SwarmingBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for SwarmingBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for SwarmingBehaviour {
    fn name(&self) -> &'static str { "SwarmingBehaviour" }

    /// Java logic (handleExecuteStepHook — two hook points):
    ///
    /// Initial phase (kickoff hook):
    ///   1. Find all reserve players on the acting team with Swarming skill.
    ///   2. Roll dice to determine the maximum number allowed to swarm this turn
    ///      (StepState.swarmingAllowedCount).
    ///   3. Show placement dialog to the coach.
    ///   4. Set StepState.turnMode = SWARMING.
    ///
    /// End-turn phase:
    ///   1. Validate the number of swarming players actually placed vs allowed count.
    ///   2. Reset any prone swarming reserves back to RESERVE status.
    ///   3. Restore StepState.turnMode = KICKOFF.
    ///
    // TODO(hook-infra): step-specific state (StepState.swarmingAllowedCount)
    // TODO(hook-infra): step-specific state (StepState.turnMode)
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = SwarmingBehaviour::new();
        assert_eq!(b.name(), "SwarmingBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = SwarmingBehaviour::default();
        assert_eq!(b.name(), "SwarmingBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = SwarmingBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2016,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = SwarmingBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
}
