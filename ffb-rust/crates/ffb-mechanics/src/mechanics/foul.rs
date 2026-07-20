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

// Java-derived cases (FoulCalcTest) live in the 1:1 mirror module
// ffb-engine/src/util/foul_calc.rs; only mechanics-specific API tests remain here.
#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;

    // ── is_foul_armour_broken (ffb-mechanics-only API, no Java counterpart) ──

    #[test]
    fn foul_armour_broken_at_av_plus_one() {
        for rules in [Rules::Bb2016, Rules::Bb2020, Rules::Bb2025] {
            assert!(is_foul_armour_broken(8, 9, rules));
            assert!(!is_foul_armour_broken(8, 8, rules));
        }
    }
}
