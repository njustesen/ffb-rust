use crate::skill_behaviour::SkillBehaviour;

/// BB2020 StandFirm skill behaviour.
///
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2020.StandFirmBehaviour`.
///
/// **BB2020 vs BB2025 differences:**
///
/// 1. **Adjacency check:** BB2020 checks that the defender with StandFirm is adjacent to the
///    attacker using coordinate arithmetic:
///    ```java
///    game.getFieldModel().getPlayerCoordinate(actingPlayer.getPlayer())
///        .isAdjacent(game.getFieldModel().getPlayerCoordinate(state.defender))
///    ```
///    BB2025 simplifies this to a team-check:
///    ```java
///    state.defender.getTeam() != actingPlayer.getPlayer().getTeam()
///    ```
///    The BB2025 condition is logically weaker (only checks opposing team, not adjacency), but
///    the adjacency check was apparently redundant in practice.
///
/// 2. **Strip-ball prevention report:** BB2025 adds `BALL_KNOCKED_LOSE = false` and
///    `CATCH_SCATTER_THROW_IN_MODE = null` step parameters plus a `ReportEvent` message when the
///    defender has the ball and stands firm. BB2020 does not emit these.
pub struct StandFirmBehaviour;

impl StandFirmBehaviour {
    pub fn new() -> Self { Self }

    /// Returns `true` when the eligibility check for StandFirm requires coordinate adjacency
    /// (BB2020), or `false` when only the opposing-team check is used (BB2025).
    pub const fn requires_adjacency_check() -> bool {
        true
    }

    /// Returns `true` when strip-ball prevention reports are emitted on StandFirm success
    /// (BB2025), `false` for BB2020 which does not emit them.
    pub const fn emits_strip_ball_prevention_report() -> bool {
        false
    }
}

impl Default for StandFirmBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for StandFirmBehaviour {
    fn name(&self) -> &'static str { "StandFirmBehaviour" }

    /// TODO(hook-infra): step-specific state access not yet wired.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// BB2020 requires coordinate-adjacency for StandFirm eligibility.
    #[test]
    fn bb2020_requires_adjacency_check() {
        assert!(StandFirmBehaviour::requires_adjacency_check());
    }

    /// BB2020 does not emit strip-ball prevention reports.
    #[test]
    fn bb2020_does_not_emit_strip_ball_prevention_report() {
        assert!(!StandFirmBehaviour::emits_strip_ball_prevention_report());
    }

    /// The two constants capture opposite behaviours between editions.
    #[test]
    fn adjacency_check_is_true_strip_ball_report_is_false() {
        assert!(StandFirmBehaviour::requires_adjacency_check());
        assert!(!StandFirmBehaviour::emits_strip_ball_prevention_report());
    }

    #[test]
    fn name_is_correct() {
        assert_eq!(StandFirmBehaviour::new().name(), "StandFirmBehaviour");
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
        let before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, before);
    }
}
