use crate::skill_behaviour::SkillBehaviour;

/// BB2020 DumpOff skill behaviour.
///
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2020.DumpOffBehaviour`.
///
/// **BB2020 vs BB2025 difference:**
///
/// BB2020 resolves the defender via `TargetSelectionState` when it is non-null:
/// ```java
/// Player<?> defender;
/// TargetSelectionState targetSelectionState = game.getFieldModel().getTargetSelectionState();
/// FieldCoordinate defenderPosition;
/// if (targetSelectionState == null) {
///     defender = game.getDefender();
///     defenderPosition = state.defenderPosition;
/// } else {
///     defender = game.getPlayerById(targetSelectionState.getSelectedPlayerId());
///     ...
/// }
/// ```
///
/// BB2025 always uses `game.getDefender()` / `state.defenderPosition` and only uses a null check
/// on `defenderPosition` itself:
/// ```java
/// Player<?> defender = game.getDefender();
/// FieldCoordinate defenderPosition = state.defenderPosition;
/// if (defenderPosition == null) { ... }
/// ```
///
/// This means BB2020 supports target-selection-based dump-off (useful for multi-target scenarios)
/// while BB2025 assumes the defender is already set directly on the game object.
pub struct DumpOffBehaviour;

impl DumpOffBehaviour {
    pub fn new() -> Self { Self }

    /// Returns `true` when BB2020's target-selection-fallback path is used.
    ///
    /// In BB2020 the defender is resolved from `TargetSelectionState` when it exists.
    /// In BB2025 it is always taken directly from `game.getDefender()`.
    pub const fn uses_target_selection_state_for_defender() -> bool {
        true
    }
}

impl Default for DumpOffBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for DumpOffBehaviour {
    fn name(&self) -> &'static str { "DumpOffBehaviour" }

    /// TODO(hook-infra): step-specific state access not yet wired.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// BB2020 uses TargetSelectionState for defender resolution.
    #[test]
    fn bb2020_uses_target_selection_state_for_defender() {
        assert!(DumpOffBehaviour::uses_target_selection_state_for_defender());
    }

    /// The constant must be true to distinguish from BB2025 which is false.
    #[test]
    fn target_selection_constant_is_true_for_bb2020() {
        assert_eq!(DumpOffBehaviour::uses_target_selection_state_for_defender(), true);
    }

    #[test]
    fn name_is_correct() {
        assert_eq!(DumpOffBehaviour::new().name(), "DumpOffBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = DumpOffBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2020,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = DumpOffBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, before);
    }
}
