use crate::skill_behaviour::SkillBehaviour;

/// BB2020 Pass skill behaviour.
///
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2020.PassBehaviour`.
///
/// **BB2020 vs BB2025 differences (StepHailMaryPass modifier):**
///
/// 1. **`SAVED_FUMBLE` treatment:** BB2020 treats `SAVED_FUMBLE` the same as `FUMBLE` for the
///    purposes of checking whether to offer a re-roll. BB2025 separates them:
///    `SAVED_FUMBLE` triggers the `handleSafePass` dialog (whether to use the SafePass skill)
///    while `FUMBLE` goes to the normal re-roll path.
///    Java (BB2020): `if (PassResult.FUMBLE == state.result || PassResult.WILDLY_INACCURATE == state.result || PassResult.SAVED_FUMBLE == state.result)`
///    Java (BB2025): `if (PassResult.FUMBLE == state.result || PassResult.SAVED_FUMBLE == state.result)` + separate `handleSafePass`
///
/// 2. **`WILDLY_INACCURATE` in re-roll guard:** BB2020 includes `WILDLY_INACCURATE` in the set
///    of results that trigger the re-roll offer. BB2025 does not include it.
///
/// 3. **`handleSafePass` method:** BB2025 adds a new method that shows the SafePass skill-use
///    dialog when the result is `SAVED_FUMBLE`. BB2020 does not have this mechanic.
///
/// 4. **`DONT_DROP_FUMBLE` parameter on `SAVED_FUMBLE`:** BB2020 sets `DONT_DROP_FUMBLE = true`
///    on a saved fumble. BB2025 defers this to the `handleSafePass` callback.
///
/// 5. **Modifying-skill / SafePass command handling:** BB2025 adds `handleCommandHook` logic for
///    `canAddStrengthToPass` and `dontDropFumbles` properties. BB2020 does not.
pub struct PassBehaviour;

impl PassBehaviour {
    pub fn new() -> Self { Self }

    /// Returns `true` when `WILDLY_INACCURATE` results should also trigger the re-roll offer
    /// in the HailMaryPass step (BB2020 behaviour).
    pub const fn wildly_inaccurate_triggers_reroll_offer() -> bool {
        true
    }

    /// Returns `true` when a `SAVED_FUMBLE` result is handled via the `handleSafePass` dialog
    /// (BB2025). BB2020 returns `false` — saved fumbles are treated like normal fumbles.
    pub const fn uses_safe_pass_dialog() -> bool {
        false
    }

    /// Classify whether a given pass result should offer a re-roll in **BB2020**.
    ///
    /// Java (BB2020):
    /// ```java
    /// if (PassResult.FUMBLE == state.result || PassResult.WILDLY_INACCURATE == state.result
    ///     || PassResult.SAVED_FUMBLE == state.result)
    /// ```
    pub fn result_triggers_reroll_offer_bb2020(result: PassResultKind) -> bool {
        matches!(
            result,
            PassResultKind::Fumble | PassResultKind::WildlyInaccurate | PassResultKind::SavedFumble
        )
    }
}

/// Pass result kinds relevant to re-roll / safe-pass handling differences.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PassResultKind {
    Accurate,
    Inaccurate,
    WildlyInaccurate,
    Fumble,
    SavedFumble,
}

impl Default for PassBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for PassBehaviour {
    fn name(&self) -> &'static str { "PassBehaviour" }

    /// TODO(hook-infra): step-specific state access not yet wired.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// BB2020: WILDLY_INACCURATE triggers re-roll offer.
    #[test]
    fn wildly_inaccurate_triggers_reroll_offer_bb2020() {
        assert!(PassBehaviour::result_triggers_reroll_offer_bb2020(PassResultKind::WildlyInaccurate));
    }

    /// BB2020: FUMBLE triggers re-roll offer.
    #[test]
    fn fumble_triggers_reroll_offer_bb2020() {
        assert!(PassBehaviour::result_triggers_reroll_offer_bb2020(PassResultKind::Fumble));
    }

    /// BB2020: SAVED_FUMBLE triggers re-roll offer (handled as fumble, no SafePass dialog).
    #[test]
    fn saved_fumble_triggers_reroll_offer_bb2020() {
        assert!(PassBehaviour::result_triggers_reroll_offer_bb2020(PassResultKind::SavedFumble));
    }

    /// BB2020 constant: wildly-inaccurate triggers re-roll offer.
    #[test]
    fn wildly_inaccurate_reroll_constant_is_true() {
        assert!(PassBehaviour::wildly_inaccurate_triggers_reroll_offer());
    }

    /// BB2020: no SafePass dialog.
    #[test]
    fn safe_pass_dialog_not_used_in_bb2020() {
        assert!(!PassBehaviour::uses_safe_pass_dialog());
    }

    /// BB2020: ACCURATE does NOT trigger re-roll offer.
    #[test]
    fn accurate_does_not_trigger_reroll_offer_bb2020() {
        assert!(!PassBehaviour::result_triggers_reroll_offer_bb2020(PassResultKind::Accurate));
    }

    #[test]
    fn name_is_correct() {
        assert_eq!(PassBehaviour::new().name(), "PassBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = PassBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2020,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = PassBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, before);
    }
}
