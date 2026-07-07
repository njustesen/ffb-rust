use crate::skill_behaviour::SkillBehaviour;

/// BB2020 SneakyGit skill behaviour. StepModifier on StepFoul: if fouler has SneakyGit, rolls 3+
/// to avoid referee ejection even on doubles. Mirrors Java
/// `com.fumbbl.ffb.server.skillbehaviour.bb2020.SneakyGitBehaviour`.
pub struct SneakyGitBehaviour;

impl SneakyGitBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for SneakyGitBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for SneakyGitBehaviour {
    fn name(&self) -> &'static str { "SneakyGitBehaviour" }

    /// Java `StepModifier<StepFoul, StepState>.handleExecuteStepHook`: if fouler has SneakyGit,
    /// rolls 3+ to avoid referee ejection even on doubles. Returns false always.
    /// TODO(hook-infra): needs step state for foul roll result and ejection flag.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        // TODO(hook-infra): step-specific state access (StepState.xxx)
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_is_noop_returns_false() {
        // Without step infra the hook always returns false.
        let b = SneakyGitBehaviour::new();
        assert_eq!(b.name(), "SneakyGitBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = SneakyGitBehaviour::default();
        assert_eq!(b.name(), "SneakyGitBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = SneakyGitBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2020,
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
#[test]    fn name_is_not_empty() {        assert!(!SneakyGitBehaviour::new().name().is_empty());    }    #[test]    fn execute_step_hook_false_with_bb2020() {        use ffb_model::enums::Rules;        use crate::step::framework::test_team;        let b = SneakyGitBehaviour::new();        let mut game = ffb_model::model::game::Game::new(            test_team("home", 0), test_team("away", 0), Rules::Bb2020,        );        assert!(!b.execute_step_hook(&mut game));    }
}
