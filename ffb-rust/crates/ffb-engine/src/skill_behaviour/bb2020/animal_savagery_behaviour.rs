use crate::skill_behaviour::SkillBehaviour;

/// BB2020 AnimalSavagery skill behaviour.
///
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2020.AnimalSavageryBehaviour`.
///
/// **BB2020 vs BB2025 differences:**
///
/// 1. **Injury mode on lash-out:** BB2020 selects the injury mode based on whether the player is
///    standing up or the target is a team-mate:
///    ```java
///    InjuryTypeBlock.Mode mode = (actingPlayer.isStandingUp() ||
///        actingPlayer.getPlayer().getTeam() != game.getDefender().getTeam())
///        ? InjuryTypeBlock.Mode.DO_NOT_USE_MODIFIERS
///        : InjuryTypeBlock.Mode.USE_MODIFIERS_AGAINST_TEAM_MATES;
///    ```
///    BB2025 always uses `USE_ARMOUR_MODIFIERS_ONLY_AGAINST_TEAM_MATES`:
///    ```java
///    InjuryTypeBlock.Mode mode = InjuryTypeBlock.Mode.USE_ARMOUR_MODIFIERS_ONLY_AGAINST_TEAM_MATES;
///    ```
///
/// 2. **Action cancellation helper:** BB2020 has a private `cancelPlayerAction(step, lashedOut)`
///    that combines `PASS`/`THROW_TEAM_MATE` under `setPassUsed`. BB2025 uses command objects
///    (`AnimalSavageryCancelActionCommand`, `AnimalSavageryControlCommand`) and separates TTM into
///    `setTtmUsed`.
///
/// 3. **`DropPlayerContext` vs `SteadyFootingContext`:** BB2020 uses `DropPlayerContext` directly;
///    BB2025 wraps it in `SteadyFootingContext` with a list of `DeferredCommand` objects.
///
/// 4. **`ANIMAL_SAVAGERY_LASH_OUT_ENDS_ACTIVATION` option:** BB2025 checks a new game option to
///    decide whether a lash-out ends the activation. BB2020 always ends it.
pub struct AnimalSavageryBehaviour;

impl AnimalSavageryBehaviour {
    pub fn new() -> Self { Self }

    /// Choose the BB2020 injury mode for an AnimalSavagery lash-out attack.
    ///
    /// - `is_standing_up`: `actingPlayer.isStandingUp()`
    /// - `target_is_team_mate`: whether the target belongs to the attacker's team
    ///
    /// Returns `true` for `DO_NOT_USE_MODIFIERS`, `false` for `USE_MODIFIERS_AGAINST_TEAM_MATES`.
    pub fn use_no_modifiers_injury_mode_bb2020(
        is_standing_up: bool,
        target_is_team_mate: bool,
    ) -> bool {
        // BB2020: no-modifier mode when standing up OR when hitting opponent (not team-mate).
        is_standing_up || !target_is_team_mate
    }

    /// Returns `true` when `THROW_TEAM_MATE` action cancellation uses `setPassUsed` (BB2020).
    /// BB2025 uses `setTtmUsed` — returns `false`.
    pub const fn ttm_cancel_uses_pass_used() -> bool {
        true
    }

    /// Returns `true` when the `ANIMAL_SAVAGERY_LASH_OUT_ENDS_ACTIVATION` game option is
    /// respected (BB2025). BB2020 always ends the activation — returns `false`.
    pub const fn respects_lash_out_ends_activation_option() -> bool {
        false
    }
}

impl Default for AnimalSavageryBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for AnimalSavageryBehaviour {
    fn name(&self) -> &'static str { "AnimalSavageryBehaviour" }

    /// TODO(hook-infra): step-specific state access not yet wired.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- BB2020 injury mode tests ---

    /// BB2020: standing-up attacker uses DO_NOT_USE_MODIFIERS.
    #[test]
    fn standing_up_uses_no_modifier_mode_bb2020() {
        assert!(AnimalSavageryBehaviour::use_no_modifiers_injury_mode_bb2020(true, true));
    }

    /// BB2020: hitting a non-team-mate uses DO_NOT_USE_MODIFIERS.
    #[test]
    fn hitting_opponent_uses_no_modifier_mode_bb2020() {
        assert!(AnimalSavageryBehaviour::use_no_modifiers_injury_mode_bb2020(false, false));
    }

    /// BB2020: not standing up and hitting a team-mate uses USE_MODIFIERS_AGAINST_TEAM_MATES.
    #[test]
    fn not_standing_and_hitting_teammate_uses_modifier_mode_bb2020() {
        assert!(!AnimalSavageryBehaviour::use_no_modifiers_injury_mode_bb2020(false, true));
    }

    // --- BB2020 action-flag tests ---

    /// BB2020: TTM cancellation uses passUsed flag.
    #[test]
    fn bb2020_ttm_cancel_uses_pass_used() {
        assert!(AnimalSavageryBehaviour::ttm_cancel_uses_pass_used());
    }

    /// BB2020: does not respect the lash-out-ends-activation option.
    #[test]
    fn bb2020_does_not_respect_lash_out_option() {
        assert!(!AnimalSavageryBehaviour::respects_lash_out_ends_activation_option());
    }

    // --- infrastructure tests ---

    #[test]
    fn name_is_correct() {
        assert_eq!(AnimalSavageryBehaviour::new().name(), "AnimalSavageryBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = AnimalSavageryBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2020,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = AnimalSavageryBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, before);
    }
}
