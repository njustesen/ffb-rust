use crate::skill_behaviour::SkillBehaviour;

/// Foul Appearance: opponents must roll 2+ before they can block the bearer.
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2016.FoulAppearanceBehaviour`.
pub struct FoulAppearanceBehaviour;

impl FoulAppearanceBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for FoulAppearanceBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for FoulAppearanceBehaviour {
    fn name(&self) -> &'static str { "FoulAppearanceBehaviour" }

    /// Java `StepModifier.handleExecuteStepHook` logic (StepBlockRoll context):
    ///
    /// 1. Roll a D6 for the attacker before they can block the defender who has Foul Appearance.
    /// 2. On a result of 1 (fail):
    ///    - Set `StepState.hasBlocked = true`.
    ///    - Set `StepState.turnStarted = true`.
    ///    - GOTO `StepState.goToLabelOnFailure` (skip remainder of block sequence).
    /// 3. On success (2+): continue normally.
    /// 4. The step supports re-rolling via `ReRollSource::FOUL_APPEARANCE` if available.
    ///
    /// All step-local state fields are unavailable in the current Rust signature:
    // TODO(hook-infra): step-specific state (StepState.goToLabelOnFailure)
    // TODO(hook-infra): step-specific state (StepState.hasBlocked)
    // TODO(hook-infra): step-specific state (StepState.turnStarted)
    // TODO(hook-infra): step reroll fields (ReRollSource::FOUL_APPEARANCE)
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = FoulAppearanceBehaviour::new();
        assert_eq!(b.name(), "FoulAppearanceBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = FoulAppearanceBehaviour::default();
        assert_eq!(b.name(), "FoulAppearanceBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = FoulAppearanceBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2016,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = FoulAppearanceBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
#[test]    fn name_is_not_empty() {        assert!(!FoulAppearanceBehaviour::new().name().is_empty());    }    #[test]    fn execute_step_hook_false_with_bb2020() {        use ffb_model::enums::Rules;        use crate::step::framework::test_team;        let b = FoulAppearanceBehaviour::new();        let mut game = ffb_model::model::game::Game::new(            test_team("home", 0), test_team("away", 0), Rules::Bb2020,        );        assert!(!b.execute_step_hook(&mut game));    }
}
