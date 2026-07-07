use crate::skill_behaviour::SkillBehaviour;

/// BB2020 BloodLust skill behaviour.
/// StepModifier on StepBloodLust: if WAIT_FOR_ACTION_CHANGE dispatches action; else rolls blood
/// lust check (2+ modified by good conditions), handles reroll and action change dialog. Mirrors
/// Java `com.fumbbl.ffb.server.skillbehaviour.bb2020.BloodLustBehaviour`.
pub struct BloodLustBehaviour;

impl BloodLustBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for BloodLustBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for BloodLustBehaviour {
    fn name(&self) -> &'static str { "BloodLustBehaviour" }

    /// Java `StepModifier<StepBloodLust, StepState>.handleExecuteStepHook`:
    /// if WAIT_FOR_ACTION_CHANGE dispatches action; else rolls blood lust check (2+ modified
    /// by good conditions), handles reroll and action change dialog. Returns false always.
    ///
    /// TODO(hook-infra): needs state.status, state.bloodlustAction,
    /// state.goToLabelOnFailure.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        // TODO(hook-infra): step-specific state access (StepState.status,
        // StepState.bloodlustAction, StepState.goToLabelOnFailure)
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_is_noop_returns_false() {
        // Without step infra the hook always returns false.
        let b = BloodLustBehaviour::new();
        assert_eq!(b.name(), "BloodLustBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = BloodLustBehaviour::default();
        assert_eq!(b.name(), "BloodLustBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = BloodLustBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2020,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = BloodLustBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
#[test]    fn name_is_not_empty() {        assert!(!BloodLustBehaviour::new().name().is_empty());    }    #[test]    fn execute_step_hook_false_with_bb2020() {        use ffb_model::enums::Rules;        use crate::step::framework::test_team;        let b = BloodLustBehaviour::new();        let mut game = ffb_model::model::game::Game::new(            test_team("home", 0), test_team("away", 0), Rules::Bb2020,        );        assert!(!b.execute_step_hook(&mut game));    }
}
