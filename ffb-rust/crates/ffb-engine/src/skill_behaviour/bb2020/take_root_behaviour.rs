use crate::skill_behaviour::SkillBehaviour;

/// BB2020 TakeRoot skill behaviour.
///
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2020.TakeRootBehaviour`.
///
/// **BB2020 vs BB2025 differences:**
///
/// 1. **`startedStanding` guard for root application:** BB2025 only applies the "rooted" state
///    if the player started the activation standing (`startedStanding = oldPlayerState.base == STANDING`).
///    BB2020 applies it unconditionally when `!playerState.isRooted()`.
///
///    Java (BB2020): `if (!playerState.isRooted()) { ... apply root ... }`
///    Java (BB2025): `if (startedStanding && !playerState.isRooted()) { ... apply root ... }`
///
/// 2. **Strip-ball prevention report on StandFirm-style rooted pushback:** BB2025 adds
///    `BALL_KNOCKED_LOSE = false`, `CATCH_SCATTER_THROW_IN_MODE = null` and a `ReportEvent`
///    message when a rooted player with the ball resists a pushback. BB2020 does not.
pub struct TakeRootBehaviour;

impl TakeRootBehaviour {
    pub fn new() -> Self { Self }

    /// Returns `true` when root application requires that the player started the activation
    /// in the standing state (BB2025 guard). BB2020 returns `false` — no such requirement.
    pub const fn requires_started_standing_for_root() -> bool {
        false
    }

    /// Returns `true` when strip-ball prevention reports are emitted when a rooted player
    /// resists a pushback (BB2025 feature). BB2020 returns `false`.
    pub const fn emits_strip_ball_prevention_report() -> bool {
        false
    }
}

impl Default for TakeRootBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for TakeRootBehaviour {
    fn name(&self) -> &'static str { "TakeRootBehaviour" }

    /// TODO(hook-infra): step-specific state access not yet wired.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// BB2020 does not require the player to have started standing before applying root.
    #[test]
    fn bb2020_does_not_require_started_standing_for_root() {
        assert!(!TakeRootBehaviour::requires_started_standing_for_root());
    }

    /// BB2020 does not emit strip-ball prevention reports.
    #[test]
    fn bb2020_does_not_emit_strip_ball_prevention_report() {
        assert!(!TakeRootBehaviour::emits_strip_ball_prevention_report());
    }

    /// Both BB2025 features are absent in BB2020.
    #[test]
    fn both_bb2025_features_absent_in_bb2020() {
        assert!(!TakeRootBehaviour::requires_started_standing_for_root());
        assert!(!TakeRootBehaviour::emits_strip_ball_prevention_report());
    }

    #[test]
    fn name_is_correct() {
        assert_eq!(TakeRootBehaviour::new().name(), "TakeRootBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = TakeRootBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2020,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = TakeRootBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, before);
    }
}
