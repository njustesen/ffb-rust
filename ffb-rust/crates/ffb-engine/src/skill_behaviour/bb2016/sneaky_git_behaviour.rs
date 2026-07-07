use crate::skill_behaviour::SkillBehaviour;

/// Sneaky Git: not automatically ejected on a double-1 foul roll.
/// Two StepModifiers: StepEjectPlayer and StepReferee.
///   - StepEjectPlayer: if ArgueTheCall succeeded → RESERVE; if SNEAKY_GIT_BAN_TO_KO option
///     → KNOCKED_OUT; else BANNED.
///   - StepReferee: foul only spotted if armour was broken.
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2016.SneakyGitBehaviour`.
pub struct SneakyGitBehaviour;

impl SneakyGitBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for SneakyGitBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for SneakyGitBehaviour {
    fn name(&self) -> &'static str { "SneakyGitBehaviour" }

    /// Java logic (two handleExecuteStepHook registrations):
    ///
    /// StepEjectPlayer hook:
    ///   1. Read StepState.argueTheCallSucceeded.
    ///   2. If ArgueTheCall succeeded: set playerStatus = RESERVE (not banned).
    ///   3. Else if game option SNEAKY_GIT_BAN_TO_KO is set: set playerStatus = KNOCKED_OUT.
    ///   4. Else: set playerStatus = BANNED (normal ejection).
    ///   5. Publish EJECTION_RESULT report.
    ///
    /// StepReferee hook:
    ///   1. Read StepState.armourBroken.
    ///   2. Foul is only spotted (referee notices) if armour was broken during the foul.
    ///   3. If armour not broken: suppress the referee spotting (return without penalty).
    ///
    // TODO(hook-infra): step-specific state (StepState.argueTheCallSucceeded)
    // TODO(hook-infra): step-specific state (StepState.armourBroken)
    // TODO(hook-infra): step-specific state (StepState.playerStatus for ejection)
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = SneakyGitBehaviour::new();
        assert_eq!(b.name(), "SneakyGitBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = SneakyGitBehaviour::default();
        assert_eq!(b.name(), "SneakyGitBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = SneakyGitBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2016,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = SneakyGitBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
}
