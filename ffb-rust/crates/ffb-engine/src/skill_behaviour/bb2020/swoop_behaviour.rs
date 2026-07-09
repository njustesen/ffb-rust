use crate::skill_behaviour::SkillBehaviour;

/// BB2020 Swoop skill behaviour.
///
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2020.SwoopBehaviour`.
///
/// **BB2020 vs BB2025 differences:**
///
/// BB2025 adds re-roll support for swoop direction and a visual indicator square:
///
/// 1. **Re-roll for direction:** BB2025 adds `SWOOP_DIRECTION` as a re-rollable action. When the
///    direction roll is available for re-roll, BB2025 shows the direction on the field, then asks
///    for a re-roll. BB2020 simply takes the first roll and publishes the direction immediately.
///
/// 2. **`usingSwoop` guard:** BB2025 wraps the scatter path in `if (state.usingSwoop && ...)`.
///    BB2020 checks only `if (swoopingPlayer.hasSkillProperty(...))` without a `usingSwoop` flag.
///
/// 3. **`state.swoopDirection` storage:** BB2025 stores the chosen direction in `state.swoopDirection`
///    so it survives a re-roll dialog cycle. BB2020 publishes the direction inline without storing.
///
/// 4. **Field indicator:** BB2025 paints a `MoveSquare` indicator on the board to show the
///    projected landing coordinate. BB2020 does not.
///
/// 5. **`ReportSwoopDirection` report:** BB2025 emits this report; BB2020 does not.
pub struct SwoopBehaviour;

impl SwoopBehaviour {
    pub fn new() -> Self { Self }

    /// Returns `true` when the Swoop direction roll supports re-rolls (BB2025 feature).
    /// BB2020 always returns `false`.
    pub const fn direction_roll_supports_reroll() -> bool {
        false
    }

    /// Returns `true` when a field indicator square is painted for the projected landing
    /// coordinate after the direction roll (BB2025 feature).
    /// BB2020 always returns `false`.
    pub const fn paints_field_indicator() -> bool {
        false
    }
}

impl Default for SwoopBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for SwoopBehaviour {
    fn name(&self) -> &'static str { "SwoopBehaviour" }

    /// TODO(hook-infra): step-specific state access not yet wired.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// BB2020 does not support direction-roll re-rolls.
    #[test]
    fn bb2020_direction_roll_does_not_support_reroll() {
        assert!(!SwoopBehaviour::direction_roll_supports_reroll());
    }

    /// BB2020 does not paint a field indicator.
    #[test]
    fn bb2020_does_not_paint_field_indicator() {
        assert!(!SwoopBehaviour::paints_field_indicator());
    }

    /// Both BB2020-specific constants are false (BB2025 features absent).
    #[test]
    fn both_bb2025_features_absent_in_bb2020() {
        assert!(!SwoopBehaviour::direction_roll_supports_reroll());
        assert!(!SwoopBehaviour::paints_field_indicator());
    }

    #[test]
    fn name_is_correct() {
        assert_eq!(SwoopBehaviour::new().name(), "SwoopBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = SwoopBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2020,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = SwoopBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, before);
    }
}
