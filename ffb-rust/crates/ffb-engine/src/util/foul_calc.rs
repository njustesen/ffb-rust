// 1:1 translation of com.fumbbl.ffb.server.util.FoulCalc

pub struct FoulCalc;

impl FoulCalc {
    pub fn new() -> Self {
        Self
    }

    /// Determine whether the referee spots a foul based on the armor roll.
    /// Referee spots the foul if the two armor dice show the same value (doubles),
    /// unless the fouler has SneakyGit (which suppresses the armor-roll detection).
    pub fn is_spotted_by_armor_roll(armor_die1: i32, armor_die2: i32, has_sneaky_git: bool) -> bool {
        (armor_die1 == armor_die2) && !has_sneaky_git
    }

    /// Determine whether the referee spots a foul based on the injury roll.
    /// When the armor was broken, doubles on the injury roll are also spotted
    /// regardless of whether the fouler has SneakyGit.
    pub fn is_spotted_by_injury_roll(injury_die1: i32, injury_die2: i32, armor_broken: bool) -> bool {
        armor_broken && (injury_die1 == injury_die2)
    }

    /// Determine whether the referee spots the foul overall.
    /// Spotted if either the armor roll or the injury roll triggered detection.
    #[allow(clippy::too_many_arguments)]
    pub fn is_spotted_by_referee(
        armor_die1: i32,
        armor_die2: i32,
        injury_die1: i32,
        injury_die2: i32,
        armor_broken: bool,
        has_sneaky_git: bool,
    ) -> bool {
        Self::is_spotted_by_armor_roll(armor_die1, armor_die2, has_sneaky_git)
            || Self::is_spotted_by_injury_roll(injury_die1, injury_die2, armor_broken)
    }

    /// Minimum armor value to break armor in a foul.
    /// Armor roll total must strictly exceed the player's AV.
    pub fn minimum_roll_to_break_armour(armour_value: i32) -> i32 {
        armour_value + 1
    }
}

impl Default for FoulCalc {
    fn default() -> Self {
        Self::new()
    }
}

// Tests mirror ffb-java/ffb-server/src/test/java/com/fumbbl/ffb/server/util/FoulCalcTest.java 1:1
#[cfg(test)]
mod tests {
    use super::*;

    // ── is_spotted_by_armor_roll ──────────────────────────────────────────────

    #[test]
    fn armor_roll_doubles_no_sneaky_git_spotted() {
        assert!(FoulCalc::is_spotted_by_armor_roll(3, 3, false));
    }

    #[test]
    fn armor_roll_doubles_with_sneaky_git_not_spotted() {
        assert!(!FoulCalc::is_spotted_by_armor_roll(3, 3, true));
    }

    #[test]
    fn armor_roll_non_doubles_not_spotted() {
        assert!(!FoulCalc::is_spotted_by_armor_roll(2, 4, false));
    }

    #[test]
    fn armor_roll_all_doubles_no_sneaky_git_spotted() {
        for d in 1..=6 {
            assert!(FoulCalc::is_spotted_by_armor_roll(d, d, false), "d1={d} d2={d}");
        }
    }

    #[test]
    fn armor_roll_all_doubles_sneaky_git_not_spotted() {
        for d in 1..=6 {
            assert!(!FoulCalc::is_spotted_by_armor_roll(d, d, true), "d1={d} d2={d}");
        }
    }

    // ── is_spotted_by_injury_roll ─────────────────────────────────────────────

    #[test]
    fn injury_roll_doubles_armor_broken_spotted() {
        assert!(FoulCalc::is_spotted_by_injury_roll(4, 4, true));
    }

    #[test]
    fn injury_roll_doubles_armor_not_broken_not_spotted() {
        assert!(!FoulCalc::is_spotted_by_injury_roll(4, 4, false));
    }

    #[test]
    fn injury_roll_non_doubles_armor_broken_not_spotted() {
        assert!(!FoulCalc::is_spotted_by_injury_roll(3, 5, true));
    }

    #[test]
    fn injury_roll_all_doubles_when_armor_broken_spotted() {
        for d in 1..=6 {
            assert!(FoulCalc::is_spotted_by_injury_roll(d, d, true), "d1={d} d2={d}");
        }
    }

    // ── is_spotted_by_referee (combined) ──────────────────────────────────────

    #[test]
    fn referee_armor_doubles_spotted() {
        // Armor doubles, armor not broken, no SneakyGit
        assert!(FoulCalc::is_spotted_by_referee(2, 2, 1, 3, false, false));
    }

    #[test]
    fn referee_injury_doubles_armor_broken_spotted() {
        // Non-double armor, armor broken, injury doubles
        assert!(FoulCalc::is_spotted_by_referee(3, 5, 4, 4, true, false));
    }

    #[test]
    fn referee_no_doubles_not_spotted() {
        // No doubles anywhere
        assert!(!FoulCalc::is_spotted_by_referee(2, 4, 3, 5, true, false));
    }

    #[test]
    fn referee_armor_doubles_sneaky_git_injury_not_doubles_not_spotted() {
        // SneakyGit suppresses armor-roll detection, injury not doubles
        assert!(!FoulCalc::is_spotted_by_referee(3, 3, 2, 5, false, true));
    }

    #[test]
    fn referee_armor_doubles_sneaky_git_injury_doubles_armor_broken_spotted() {
        // SneakyGit suppresses armor-roll detection, but injury doubles fires
        assert!(FoulCalc::is_spotted_by_referee(3, 3, 4, 4, true, true));
    }

    #[test]
    fn referee_no_armor_roll_injury_doubles_armor_not_broken_not_spotted() {
        // Armor not broken (no injury roll made), doubles on injury ignored
        assert!(!FoulCalc::is_spotted_by_referee(2, 4, 3, 3, false, false));
    }

    // ── minimum_roll_to_break_armour ──────────────────────────────────────────

    #[test]
    fn minimum_roll_to_break_armour_av_plus_one() {
        for (av, expected) in [(7, 8), (8, 9), (9, 10), (10, 11), (11, 12)] {
            assert_eq!(expected, FoulCalc::minimum_roll_to_break_armour(av), "av={av}");
        }
    }
}
