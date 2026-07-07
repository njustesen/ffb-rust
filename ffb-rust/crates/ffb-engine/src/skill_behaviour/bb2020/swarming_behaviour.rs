use crate::skill_behaviour::SkillBehaviour;

/// BB2020 Swarming skill behaviour. StepModifier on StepSwarming: if in SWARMING turn mode handles
/// reserve selection and placement; otherwise checks if team has swarming reserves and shows
/// dialog. Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2020.SwarmingBehaviour`.
pub struct SwarmingBehaviour;

impl SwarmingBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for SwarmingBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for SwarmingBehaviour {
    fn name(&self) -> &'static str { "SwarmingBehaviour" }

    /// Java `StepModifier<StepSwarming, StepState>.handleExecuteStepHook`: if TurnMode==SWARMING
    /// handles endTurn or player placement; else checks reserves with swarming skill and shows
    /// dialog. Returns false always.
    /// TODO(hook-infra): needs state.endTurn, state.swarmingPlayers, game.getTurnMode().
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
        let b = SwarmingBehaviour::new();
        assert_eq!(b.name(), "SwarmingBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = SwarmingBehaviour::default();
        assert_eq!(b.name(), "SwarmingBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = SwarmingBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2020,
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
#[test]    fn name_is_not_empty() {        assert!(!SwarmingBehaviour::new().name().is_empty());    }    #[test]    fn execute_step_hook_false_with_bb2020() {        use ffb_model::enums::Rules;        use crate::step::framework::test_team;        let b = SwarmingBehaviour::new();        let mut game = ffb_model::model::game::Game::new(            test_team("home", 0), test_team("away", 0), Rules::Bb2020,        );        assert!(!b.execute_step_hook(&mut game));    }
}
