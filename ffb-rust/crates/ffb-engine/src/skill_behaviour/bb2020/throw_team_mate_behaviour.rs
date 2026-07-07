use crate::skill_behaviour::SkillBehaviour;

/// BB2020 ThrowTeamMate skill behaviour. StepModifier on StepThrowTeamMate: rolls throw-team-mate
/// pass roll, evaluates distance, handles fumble with reroll dialog. Mirrors Java
/// `com.fumbbl.ffb.server.skillbehaviour.bb2020.ThrowTeamMateBehaviour`.
pub struct ThrowTeamMateBehaviour;

impl ThrowTeamMateBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for ThrowTeamMateBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for ThrowTeamMateBehaviour {
    fn name(&self) -> &'static str { "ThrowTeamMateBehaviour" }

    /// Java `StepModifier<StepThrowTeamMate, StepState>.handleExecuteStepHook`: rolls
    /// throw-team-mate pass roll, evaluates distance, handles fumble with reroll dialog. Returns
    /// false always.
    /// TODO(hook-infra): needs state for pass roll, distance, fumble.
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
        let b = ThrowTeamMateBehaviour::new();
        assert_eq!(b.name(), "ThrowTeamMateBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = ThrowTeamMateBehaviour::default();
        assert_eq!(b.name(), "ThrowTeamMateBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = ThrowTeamMateBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2020,
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
#[test]    fn name_is_not_empty() {        assert!(!ThrowTeamMateBehaviour::new().name().is_empty());    }    #[test]    fn execute_step_hook_false_with_bb2020() {        use ffb_model::enums::Rules;        use crate::step::framework::test_team;        let b = ThrowTeamMateBehaviour::new();        let mut game = ffb_model::model::game::Game::new(            test_team("home", 0), test_team("away", 0), Rules::Bb2020,        );        assert!(!b.execute_step_hook(&mut game));    }
}
