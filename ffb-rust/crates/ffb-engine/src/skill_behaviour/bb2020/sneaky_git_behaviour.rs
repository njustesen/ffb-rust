use crate::skill_behaviour::SkillBehaviour;

/// BB2020 SneakyGit skill behaviour.
///
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2020.SneakyGitBehaviour`.
///
/// **BB2020 vs BB2025 differences (StepReferee modifier):**
///
/// 1. Armor-roll doubles detection: In BB2020 the armor-roll doubles only trigger a referee spot
///    when the fouler does NOT have SneakyGit (the condition appends
///    `&& !UtilCards.hasSkill(actingPlayer, skill)`). In BB2025 that guard is absent, so SneakyGit
///    holders can still be spotted via the armor-roll path if armor was broken.
///
/// 2. Under-scrutiny flag: In BB2020 `refereeSpotsFoul |= underScrutiny` (unconditionally sets the
///    flag whenever the team is under scrutiny). In BB2025 the flag is only set when armor was
///    broken: `refereeSpotsFoul |= (underScrutiny && isArmorBroken)`.
///
/// The StepEjectPlayer modifier is identical between editions.
pub struct SneakyGitBehaviour;

impl SneakyGitBehaviour {
    pub fn new() -> Self { Self }

    /// Compute whether the referee spots a foul using **BB2020 rules**.
    ///
    /// Parameters map directly to the Java `StepModifier<StepReferee, StepState>` fields:
    /// - `has_sneaky_git`: fouler has SneakyGit skill.
    /// - `foul_breaks_armour_without_roll`: `NamedProperties.foulBreaksArmourWithoutRoll` active.
    /// - `sneaky_git_ban_to_ko_option_enabled`: game option `SNEAKY_GIT_BAN_TO_KO`.
    /// - `armor_broken`: armor was broken on the injury roll.
    /// - `armor_roll`: the two armor dice (used to detect doubles).
    /// - `injury_roll`: the two injury dice (used to detect doubles when armor broken).
    /// - `under_scrutiny`: prayer state under scrutiny for the fouling team.
    ///
    /// Returns `true` when the referee spots the foul.
    pub fn referee_spots_foul_bb2020(
        has_sneaky_git: bool,
        foul_breaks_armour_without_roll: bool,
        sneaky_git_ban_to_ko_option_enabled: bool,
        armor_broken: bool,
        armor_roll: [u8; 2],
        injury_roll: [u8; 2],
        under_scrutiny: bool,
    ) -> bool {
        let mut referee_spots_foul = false;

        // BB2020: armor roll doubles only trigger ref-spot when fouler does NOT have SneakyGit,
        // OR when SneakyGit is present but armor was broken,
        // OR when the SNEAKY_GIT_BAN_TO_KO option is active.
        if !foul_breaks_armour_without_roll
            && (!has_sneaky_git
                || armor_broken
                || (has_sneaky_git && sneaky_git_ban_to_ko_option_enabled))
        {
            // BB2020 extra guard: armor-roll doubles only count when fouler lacks SneakyGit.
            if !has_sneaky_git {
                referee_spots_foul = armor_roll[0] == armor_roll[1];
            }
        }

        if !referee_spots_foul && armor_broken {
            referee_spots_foul = injury_roll[0] == injury_roll[1];
        }

        // BB2020: under-scrutiny unconditionally sets the ref-spots flag.
        referee_spots_foul |= under_scrutiny;

        referee_spots_foul
    }
}

impl Default for SneakyGitBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for SneakyGitBehaviour {
    fn name(&self) -> &'static str { "SneakyGitBehaviour" }

    /// TODO(hook-infra): step-specific state access not yet wired. The pure-logic helper
    /// `referee_spots_foul_bb2020` encodes the BB2020 referee-detection algorithm.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- pure logic tests for BB2020 referee detection ---

    /// BB2020: armor-roll doubles do NOT spot foul when fouler has SneakyGit and armor was NOT broken.
    #[test]
    fn armor_doubles_do_not_spot_when_sneaky_git_and_armor_not_broken() {
        let spotted = SneakyGitBehaviour::referee_spots_foul_bb2020(
            true,  // has_sneaky_git
            false, // foul_breaks_armour_without_roll
            false, // sneaky_git_ban_to_ko_option_enabled
            false, // armor_broken
            [3, 3], // armor doubles
            [1, 2], // no injury doubles
            false, // under_scrutiny
        );
        assert!(!spotted, "BB2020: SneakyGit should prevent armor-doubles ref-spot");
    }

    /// BB2020: injury-roll doubles DO spot foul even with SneakyGit when armor is broken.
    #[test]
    fn injury_doubles_spot_foul_when_armor_broken_even_with_sneaky_git() {
        let spotted = SneakyGitBehaviour::referee_spots_foul_bb2020(
            true,  // has_sneaky_git
            false, // foul_breaks_armour_without_roll
            false, // sneaky_git_ban_to_ko_option_enabled
            true,  // armor_broken
            [2, 3], // no armor doubles
            [4, 4], // injury doubles
            false, // under_scrutiny
        );
        assert!(spotted, "BB2020: injury doubles should still spot foul when armor broken");
    }

    /// BB2020: under-scrutiny unconditionally spots foul regardless of roll results.
    #[test]
    fn under_scrutiny_always_spots_foul_bb2020() {
        let spotted = SneakyGitBehaviour::referee_spots_foul_bb2020(
            true,  // has_sneaky_git
            false, // foul_breaks_armour_without_roll
            false, // sneaky_git_ban_to_ko_option_enabled
            false, // armor_broken
            [1, 2], // no armor doubles
            [3, 4], // no injury doubles
            true,  // under_scrutiny - unconditionally sets flag in BB2020
        );
        assert!(spotted, "BB2020: under-scrutiny must unconditionally spot the foul");
    }

    /// Without SneakyGit, armor-roll doubles do spot the foul normally.
    #[test]
    fn armor_doubles_spot_foul_without_sneaky_git() {
        let spotted = SneakyGitBehaviour::referee_spots_foul_bb2020(
            false, // no sneaky_git
            false, false, false,
            [5, 5], // armor doubles
            [1, 2],
            false,
        );
        assert!(spotted, "BB2020: armor doubles should spot foul without SneakyGit");
    }

    // --- infrastructure tests ---

    #[test]
    fn name_is_correct() {
        assert_eq!(SneakyGitBehaviour::new().name(), "SneakyGitBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = SneakyGitBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2020,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = SneakyGitBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
}
