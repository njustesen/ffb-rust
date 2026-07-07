use crate::skill_behaviour::SkillBehaviour;

/// BB2020 StandFirm skill behaviour. StepModifier on StepPushback: if defender has StandFirm and
/// it is not cancelled, shows choice dialog to stay in place. Mirrors Java
/// `com.fumbbl.ffb.server.skillbehaviour.bb2020.StandFirmBehaviour`.
pub struct StandFirmBehaviour;

impl StandFirmBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for StandFirmBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for StandFirmBehaviour {
    fn name(&self) -> &'static str { "StandFirmBehaviour" }

    /// Java `StepModifier<StepPushback, StepState>.handleExecuteStepHook`: if defender has
    /// StandFirm (and no cancelling skill), shows dialog for defender to opt out of pushback.
    /// Returns false always.
    /// TODO(hook-infra): needs state.defender, step dialog infra.
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
        let b = StandFirmBehaviour::new();
        assert_eq!(b.name(), "StandFirmBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = StandFirmBehaviour::default();
        assert_eq!(b.name(), "StandFirmBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = StandFirmBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2020,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = StandFirmBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
}
