use crate::skill_behaviour::SkillBehaviour;

/// BB2020 ThrowTeamMate skill behaviour.
///
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2020.ThrowTeamMateBehaviour`.
///
/// **BB2020 vs BB2025 differences:**
///
/// 1. **`setPassUsed` vs `setTtmUsed`:** BB2020 marks `passUsed` when a TTM action is cancelled.
///    BB2025 marks `ttmUsed` (a separate flag introduced for TTM/KTM tracking).
///    Java (BB2020): `game.getTurnData().setPassUsed(true);`
///    Java (BB2025): `game.getTurnData().setTtmUsed(true);`
///
/// 2. **Wildly-inaccurate result vs fumble:** BB2020 returns `WILDLY_INACCURATE` for a very bad
///    TTM throw. BB2025 returns `FUMBLE` for the same case.
///    Java (BB2020): `return PassResult.WILDLY_INACCURATE;`
///    Java (BB2025): `return PassResult.FUMBLE;`
///
/// 3. **No Bullseye dialog:** BB2025 adds a dialog for the `canSkipTtmScatterOnSuperbThrow`
///    (Bullseye) property on accurate throws. BB2020 does not have this mechanic.
///
/// 4. **No `buildTtmRerollMessage`:** BB2025 builds a custom re-roll dialog message with superb
///    target numbers for inaccurate throws. BB2020 uses the default re-roll dialog.
pub struct ThrowTeamMateBehaviour;

impl ThrowTeamMateBehaviour {
    pub fn new() -> Self { Self }

    /// Returns the pass-result variant for an extremely bad throw in **BB2020**.
    ///
    /// BB2020: `WILDLY_INACCURATE`. BB2025: `FUMBLE`.
    pub fn bad_throw_result_bb2020() -> TtmPassResult {
        TtmPassResult::WildlyInaccurate
    }

    /// Returns `true` when the Bullseye (`canSkipTtmScatterOnSuperbThrow`) dialog is shown
    /// on accurate throws. BB2020 returns `false`; BB2025 returns `true`.
    pub const fn shows_bullseye_dialog() -> bool {
        false
    }

    /// Returns `true` when TTM action cancellation marks `ttmUsed` (BB2025).
    /// BB2020 marks `passUsed` instead — returns `false`.
    pub const fn uses_ttm_used_flag() -> bool {
        false
    }
}

/// Pass result variants relevant to TTM behaviour differences.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TtmPassResult {
    Accurate,
    Inaccurate,
    WildlyInaccurate,
    Fumble,
}

impl Default for ThrowTeamMateBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for ThrowTeamMateBehaviour {
    fn name(&self) -> &'static str { "ThrowTeamMateBehaviour" }

    /// TODO(hook-infra): step-specific state access not yet wired.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// BB2020: bad throw returns WILDLY_INACCURATE (not FUMBLE as in BB2025).
    #[test]
    fn bb2020_bad_throw_is_wildly_inaccurate() {
        assert_eq!(
            ThrowTeamMateBehaviour::bad_throw_result_bb2020(),
            TtmPassResult::WildlyInaccurate
        );
    }

    /// BB2020 does not show Bullseye dialog.
    #[test]
    fn bb2020_does_not_show_bullseye_dialog() {
        assert!(!ThrowTeamMateBehaviour::shows_bullseye_dialog());
    }

    /// BB2020 uses passUsed flag (not ttmUsed).
    #[test]
    fn bb2020_uses_pass_used_flag_not_ttm_used() {
        assert!(!ThrowTeamMateBehaviour::uses_ttm_used_flag());
    }

    /// Bad throw result is not FUMBLE in BB2020.
    #[test]
    fn bb2020_bad_throw_is_not_fumble() {
        assert_ne!(
            ThrowTeamMateBehaviour::bad_throw_result_bb2020(),
            TtmPassResult::Fumble
        );
    }

    #[test]
    fn name_is_correct() {
        assert_eq!(ThrowTeamMateBehaviour::new().name(), "ThrowTeamMateBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = ThrowTeamMateBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2020,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = ThrowTeamMateBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, before);
    }
}
