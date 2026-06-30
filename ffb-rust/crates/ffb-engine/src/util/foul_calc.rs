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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doubles_without_sneaky_git_spotted() {
        assert!(FoulCalc::is_spotted_by_armor_roll(3, 3, false));
    }

    #[test]
    fn doubles_with_sneaky_git_not_spotted() {
        assert!(!FoulCalc::is_spotted_by_armor_roll(3, 3, true));
    }

    #[test]
    fn non_doubles_not_spotted() {
        assert!(!FoulCalc::is_spotted_by_armor_roll(3, 4, false));
        assert!(!FoulCalc::is_spotted_by_armor_roll(3, 4, true));
    }

    #[test]
    fn injury_roll_doubles_with_armor_broken_spotted() {
        assert!(FoulCalc::is_spotted_by_injury_roll(4, 4, true));
    }

    #[test]
    fn injury_roll_doubles_without_armor_broken_not_spotted() {
        assert!(!FoulCalc::is_spotted_by_injury_roll(4, 4, false));
    }

    #[test]
    fn injury_roll_non_doubles_not_spotted() {
        assert!(!FoulCalc::is_spotted_by_injury_roll(3, 4, true));
    }

    #[test]
    fn spotted_by_referee_armor_doubles() {
        // sneaky git does not apply to injury roll
        assert!(FoulCalc::is_spotted_by_referee(3, 3, 1, 2, false, false));
    }

    #[test]
    fn spotted_by_referee_injury_doubles_overrides_sneaky_git() {
        // Sneaky git suppresses armor detection, but injury detection still fires
        assert!(FoulCalc::is_spotted_by_referee(3, 4, 4, 4, true, true));
    }

    #[test]
    fn not_spotted_when_no_doubles_anywhere() {
        assert!(!FoulCalc::is_spotted_by_referee(3, 4, 2, 5, false, false));
    }

    #[test]
    fn minimum_roll_to_break_armour_8() {
        assert_eq!(FoulCalc::minimum_roll_to_break_armour(8), 9);
    }

    #[test]
    fn minimum_roll_to_break_armour_boundary() {
        assert_eq!(FoulCalc::minimum_roll_to_break_armour(7), 8);
        assert_eq!(FoulCalc::minimum_roll_to_break_armour(9), 10);
    }
}
