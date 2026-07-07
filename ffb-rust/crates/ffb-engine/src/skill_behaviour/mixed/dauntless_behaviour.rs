use crate::skill_behaviour::SkillBehaviour;

/// Dauntless: can attempt to reduce the opponent ST for a block (multi-edition).
///
/// Two step modifiers in Java:
///
/// **Modifier 1 — StepDauntless (single-block path):**
/// 1. Roll the Dauntless die.
/// 2. On success: publish `ReportId::SUCCESSFUL_DAUNTLESS`; if the player has Indomitable
///    and a re-roll is available, show the Indomitable dialog.
/// 3. On failure: ask for a team re-roll if available (shows re-roll dialog).
/// 4. Advance state via `StepState.status`.
///
/// **Modifier 2 — StepDauntlessMultiple (multi-block path):**
/// - Uses the `AbstractStepModifierMultipleBlock` pattern: on first run rolls each
///   block target that `requiresRoll()`, collects re-roll availability, shows dialog
///   or goes `NEXT_STEP`; on second run applies the chosen re-roll.
///
/// All step-local state fields are unavailable in the current Rust signature:
// TODO(hook-infra): step-specific state (StepState.status)
// TODO(hook-infra): step reroll fields (StepState.reRollTarget, StepState.reRollSource)
// TODO(hook-infra): step-specific state (StepState.firstRun, StepState.blockTargets)
///
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.mixed.DauntlessBehaviour`.
pub struct DauntlessBehaviour;

impl DauntlessBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for DauntlessBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for DauntlessBehaviour {
    fn name(&self) -> &'static str { "DauntlessBehaviour" }

    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = DauntlessBehaviour::new();
        assert_eq!(b.name(), "DauntlessBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = DauntlessBehaviour::default();
        assert_eq!(b.name(), "DauntlessBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = DauntlessBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2025,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = DauntlessBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
}
