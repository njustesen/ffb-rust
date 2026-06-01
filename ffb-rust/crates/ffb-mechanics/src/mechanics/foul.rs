use ffb_model::enums::Rules;

/// Returns true if the referee spots the foul based on the armor roll.
/// Doubles on armor dice are spotted unless the fouler has SneakyGit.
pub fn is_spotted_by_armor_roll(die1: i32, die2: i32, has_sneaky_git: bool) -> bool {
    (die1 == die2) && !has_sneaky_git
}

/// Returns true if the referee spots the foul based on the injury roll.
/// Doubles on injury dice are spotted when armor was broken (injury roll was made).
pub fn is_spotted_by_injury_roll(die1: i32, die2: i32, armor_broken: bool) -> bool {
    armor_broken && (die1 == die2)
}

/// Returns true if the referee spots the foul overall (either check triggers).
pub fn is_spotted_by_referee(
    armor_die1: i32,
    armor_die2: i32,
    injury_die1: i32,
    injury_die2: i32,
    armor_broken: bool,
    has_sneaky_git: bool,
) -> bool {
    is_spotted_by_armor_roll(armor_die1, armor_die2, has_sneaky_git)
        || is_spotted_by_injury_roll(injury_die1, injury_die2, armor_broken)
}

/// Minimum 2D6 total needed to break armor (AV + 1).
pub fn minimum_roll_to_break_armour(armour_value: i32) -> i32 {
    armour_value + 1
}

/// Whether the foul armor roll breaks armor (ignores _rules for now — same formula all editions).
pub fn is_foul_armour_broken(armour_value: i32, roll_total: i32, _rules: Rules) -> bool {
    roll_total >= minimum_roll_to_break_armour(armour_value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;

    // ── is_spotted_by_armor_roll ──────────────────────────────────────────────

    #[test]
    fn armor_doubles_no_sneaky_git_spotted() {
        assert!(is_spotted_by_armor_roll(3, 3, false));
    }

    #[test]
    fn armor_doubles_with_sneaky_git_not_spotted() {
        assert!(!is_spotted_by_armor_roll(3, 3, true));
    }

    #[test]
    fn armor_non_doubles_not_spotted() {
        assert!(!is_spotted_by_armor_roll(2, 4, false));
    }

    #[test]
    fn armor_all_doubles_spotted() {
        for d in 1..=6 {
            assert!(is_spotted_by_armor_roll(d, d, false), "die={}", d);
        }
    }

    #[test]
    fn armor_all_doubles_sneaky_git_not_spotted() {
        for d in 1..=6 {
            assert!(!is_spotted_by_armor_roll(d, d, true), "die={}", d);
        }
    }

    // ── is_spotted_by_injury_roll ─────────────────────────────────────────────

    #[test]
    fn injury_doubles_armor_broken_spotted() {
        assert!(is_spotted_by_injury_roll(4, 4, true));
    }

    #[test]
    fn injury_doubles_armor_not_broken_not_spotted() {
        assert!(!is_spotted_by_injury_roll(4, 4, false));
    }

    #[test]
    fn injury_non_doubles_armor_broken_not_spotted() {
        assert!(!is_spotted_by_injury_roll(3, 5, true));
    }

    #[test]
    fn injury_all_doubles_armor_broken_spotted() {
        for d in 1..=6 {
            assert!(is_spotted_by_injury_roll(d, d, true), "die={d}");
        }
    }

    // ── is_spotted_by_referee ─────────────────────────────────────────────────

    #[test]
    fn referee_armor_doubles_spotted() {
        assert!(is_spotted_by_referee(2, 2, 1, 3, false, false));
    }

    #[test]
    fn referee_injury_doubles_armor_broken_spotted() {
        assert!(is_spotted_by_referee(3, 5, 4, 4, true, false));
    }

    #[test]
    fn referee_no_doubles_not_spotted() {
        assert!(!is_spotted_by_referee(2, 4, 3, 5, true, false));
    }

    #[test]
    fn referee_armor_doubles_sneaky_git_injury_not_doubles_not_spotted() {
        assert!(!is_spotted_by_referee(3, 3, 2, 5, false, true));
    }

    #[test]
    fn referee_armor_doubles_sneaky_git_injury_doubles_armor_broken_spotted() {
        assert!(is_spotted_by_referee(3, 3, 4, 4, true, true));
    }

    #[test]
    fn referee_no_armor_roll_injury_doubles_armor_not_broken_not_spotted() {
        // Armor not broken (no injury roll made), doubles on injury ignored
        assert!(!is_spotted_by_referee(2, 4, 3, 3, false, false));
    }

    // ── minimum_roll_to_break_armour ──────────────────────────────────────────

    #[test]
    fn minimum_roll_av_plus_one() {
        for av in 7..=11 {
            assert_eq!(av + 1, minimum_roll_to_break_armour(av));
        }
    }

    // ── is_foul_armour_broken ─────────────────────────────────────────────────

    #[test]
    fn foul_armour_broken_at_av_plus_one() {
        for rules in [Rules::Bb2016, Rules::Bb2020, Rules::Bb2025] {
            assert!(is_foul_armour_broken(8, 9, rules));
            assert!(!is_foul_armour_broken(8, 8, rules));
        }
    }
}
