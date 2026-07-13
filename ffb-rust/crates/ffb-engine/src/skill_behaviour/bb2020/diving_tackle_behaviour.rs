use crate::skill_behaviour::SkillBehaviour;

/// BB2020 DivingTackle skill behaviour.
///
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2020.DivingTackleBehaviour`.
///
/// **BB2020 vs BB2025 difference:**
///
/// BB2020 uses `UtilPlayer.findAdjacentOpposingPlayersWithProperty(game, coordinateFrom,
/// NamedProperties.canAttemptToTackleDodgingPlayer, false)` followed by
/// `UtilPlayer.filterThrower` and optionally `filterAttackerAndDefender` for DUMP_OFF mode.
///
/// BB2025 replaces that with a single call to `UtilPlayer.findEligibleDivingTacklers(game,
/// coordinateFrom, coordinateTo, NamedProperties.canAttemptToTackleDodgingPlayer)` which
/// internally handles both filtering and the additional eligibility check against `coordinateTo`.
///
/// In practice, BB2025 is stricter: a potential diving tackler must also be adjacent to the
/// destination square (`coordinateTo`), not just the origin square. BB2020 only checks adjacency
/// to the origin square (`coordinateFrom`).
pub struct DivingTackleBehaviour;

impl DivingTackleBehaviour {
    pub fn new() -> Self { Self }

    /// Returns whether the BB2020 diving-tackler search requires checking adjacency to the
    /// *destination* square as well as the origin square.
    ///
    /// BB2020: `false` — only `coordinateFrom` adjacency is checked.
    /// BB2025: `true` — `findEligibleDivingTacklers` checks both squares.
    pub const fn checks_destination_adjacency() -> bool {
        false
    }
}

impl Default for DivingTackleBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for DivingTackleBehaviour {
    fn name(&self) -> &'static str { "DivingTackleBehaviour" }

    /// Dead stub (Phase AAJ): the BB2020 eligibility-lookup difference documented above is
    /// ported for real in `UtilPlayer::find_diving_tacklers`/`find_eligible_diving_tacklers`
    /// (`ffb-model/src/util/util_player.rs`), and the full dodge-math/dialog logic lives
    /// directly in `step/action/move_/step_diving_tackle.rs` (`execute_step_stat_edition`),
    /// matching the established Wrestle/Stab/DumpOff/Dauntless direct-in-step convention. This
    /// `skill_behaviour/` hook is never reached (not wired through
    /// `dispatch::execute_step_hooks`) and stays a harmless registered no-op.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// BB2020 does not check destination adjacency.
    #[test]
    fn bb2020_does_not_check_destination_adjacency() {
        assert!(!DivingTackleBehaviour::checks_destination_adjacency());
    }

    /// BB2025 would check destination adjacency — confirm the constant distinguishes editions.
    #[test]
    fn edition_constant_is_false_for_bb2020() {
        assert_eq!(DivingTackleBehaviour::checks_destination_adjacency(), false);
    }

    #[test]
    fn name_is_correct() {
        assert_eq!(DivingTackleBehaviour::new().name(), "DivingTackleBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = DivingTackleBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2020,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = DivingTackleBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, before);
    }
}
