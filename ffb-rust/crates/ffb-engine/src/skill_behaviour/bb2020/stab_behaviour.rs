use crate::skill_behaviour::SkillBehaviour;

/// BB2020 Stab skill behaviour.
///
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2020.StabBehaviour`.
///
/// **BB2020 vs BB2025 difference:**
///
/// BB2025 checks whether the acting player has `NamedProperties.grantsSppFromSpecialActionsCas`
/// and, if so, uses `InjuryTypeStabForSpp` instead of `InjuryTypeStab`. This allows certain
/// star-player Stab variants to earn SPP from stab casualties.
///
/// BB2020 always uses `InjuryTypeStab(true)` unconditionally — there is no SPP-from-stab path.
///
/// Java (BB2020):
/// ```java
/// InjuryResult injuryResultDefender = UtilServerInjury.handleInjury(
///     step, new InjuryTypeStab(true), actingPlayer.getPlayer(), ...);
/// ```
///
/// Java (BB2025):
/// ```java
/// boolean grantsSpp = UtilCards.hasSkillWithProperty(actingPlayer.getPlayer(),
///     NamedProperties.grantsSppFromSpecialActionsCas);
/// InjuryResult injuryResultDefender = UtilServerInjury.handleInjury(
///     step, grantsSpp ? new InjuryTypeStabForSpp(true) : new InjuryTypeStab(true), ...);
/// ```
pub struct StabBehaviour;

impl StabBehaviour {
    pub fn new() -> Self { Self }

    /// Returns `true` when the given player's skill properties allow the SPP-from-stab injury type.
    ///
    /// BB2020 always returns `false` — SPP-from-stab is a BB2025-only feature.
    pub fn use_stab_for_spp(_has_grants_spp_from_special_actions_cas: bool) -> bool {
        // BB2020: always use InjuryTypeStab, never InjuryTypeStabForSpp.
        false
    }
}

impl Default for StabBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for StabBehaviour {
    fn name(&self) -> &'static str { "StabBehaviour" }

    /// TODO(hook-infra): step-specific state access not yet wired.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// BB2020: never uses the SPP stab injury type, even when the property is present.
    #[test]
    fn bb2020_never_uses_stab_for_spp_when_property_present() {
        assert!(!StabBehaviour::use_stab_for_spp(true));
    }

    /// BB2020: never uses the SPP stab injury type when the property is absent.
    #[test]
    fn bb2020_never_uses_stab_for_spp_when_property_absent() {
        assert!(!StabBehaviour::use_stab_for_spp(false));
    }

    /// Both cases return false — BB2020 is edition-invariant on this point.
    #[test]
    fn stab_for_spp_always_false_in_bb2020() {
        for has_prop in [true, false] {
            assert_eq!(
                StabBehaviour::use_stab_for_spp(has_prop),
                false,
                "BB2020 must never use InjuryTypeStabForSpp"
            );
        }
    }

    #[test]
    fn name_is_correct() {
        assert_eq!(StabBehaviour::new().name(), "StabBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = StabBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2020,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = StabBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, before);
    }
}
