use crate::skill_behaviour::SkillBehaviour;

/// Juggernaut: removes Wrestle/Sidestep/Stand Firm from block results on a Blitz action
/// (multi-edition).
///
/// Registers on StepJuggernaut.
///
/// Java `execute_step_hook` logic:
/// 1. Only acts when the current action is `BLITZ` and the player has the Juggernaut skill.
/// 2. If `StepState.usingJuggernaut` is null:
///    - Show the skill-use dialog to the active coach and return `true` (waiting).
/// 3. If `StepState.usingJuggernaut == true`:
///    - Publish `ReportId::BLOCK_RESULT` with value `PUSHBACK`.
///    - Restore `StepState.oldDefenderState` (undo any Wrestle/Sidestep/Stand Firm effect).
///    - Initialise pushback.
///    - GOTO `StepState.goToLabelOnSuccess`.
/// 4. If `StepState.usingJuggernaut == false`:
///    - Advance to `NEXT_STEP`.
///
/// All step-local state fields are unavailable in the current Rust signature:
// TODO(hook-infra): step-specific state (StepState.usingJuggernaut)
// TODO(hook-infra): step-specific state (StepState.goToLabelOnSuccess)
// TODO(hook-infra): step-specific state (StepState.oldDefenderState)
///
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.mixed.JuggernautBehaviour`.
pub struct JuggernautBehaviour;

impl JuggernautBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for JuggernautBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for JuggernautBehaviour {
    fn name(&self) -> &'static str { "JuggernautBehaviour" }

    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = JuggernautBehaviour::new();
        assert_eq!(b.name(), "JuggernautBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = JuggernautBehaviour::default();
        assert_eq!(b.name(), "JuggernautBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = JuggernautBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2025,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = JuggernautBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
}
