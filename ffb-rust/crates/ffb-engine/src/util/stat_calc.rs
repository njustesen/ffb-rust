// 1:1 translation of com.fumbbl.ffb.server.util.StatCalc
//
// Pure player stat limit and lasting-injury calculations.
// Mirrors Java StatsMechanic implementations for each edition.

use ffb_model::enums::{PlayerStatKey, Rules};

pub struct StatCalc;

impl StatCalc {
    pub fn new() -> Self {
        Self
    }

    // ── Stat limits ───────────────────────────────────────────────────────────

    /// Minimum allowed value for the given stat in the given edition.
    /// BB2016: all stats minimum 1.
    /// BB2020/BB2025: AV minimum 3; all others minimum 1.
    pub fn stat_min(key: PlayerStatKey, rules: Rules) -> i32 {
        if rules == Rules::Bb2020 || rules == Rules::Bb2025 {
            if key == PlayerStatKey::Av { 3 } else { 1 }
        } else {
            1
        }
    }

    /// Maximum allowed value for the given stat in the given edition.
    /// BB2016: all stats maximum 10.
    /// BB2020/BB2025: MA=9, ST=8, AG=6, PA=6, AV=11.
    pub fn stat_max(key: PlayerStatKey, rules: Rules) -> i32 {
        if rules == Rules::Bb2020 || rules == Rules::Bb2025 {
            match key {
                PlayerStatKey::Ma => 9,
                PlayerStatKey::St => 8,
                PlayerStatKey::Ag => 6,
                PlayerStatKey::Pa => 6,
                PlayerStatKey::Av => 11,
            }
        } else {
            // BB2016: all stats cap at 10
            match key {
                PlayerStatKey::Ma
                | PlayerStatKey::St
                | PlayerStatKey::Ag
                | PlayerStatKey::Av => 10,
                PlayerStatKey::Pa => 0,
            }
        }
    }

    /// Apply a lasting injury (post-game) to a stat value, clamped to edition limits.
    ///
    /// BB2016: all stats -1 (floored at minimum).
    /// BB2020/BB2025: AG and PA +1 (worse target number, capped at max); all others -1.
    pub fn apply_lasting_injury(value: i32, key: PlayerStatKey, rules: Rules) -> i32 {
        let min = Self::stat_min(key, rules);
        let max = Self::stat_max(key, rules);
        if rules == Rules::Bb2020 || rules == Rules::Bb2025 {
            if key == PlayerStatKey::Ag || key == PlayerStatKey::Pa {
                return (value + 1).min(max);
            }
            return (value - 1).max(min);
        }
        // BB2016: all decrease
        (value - 1).max(min)
    }

    /// Apply an in-game agility injury (e.g. from Niggling Injury) to the current agility value.
    /// BB2016: agility decreases (higher = better in BB2016).
    /// BB2020/BB2025: agility target number increases (higher = worse).
    pub fn apply_in_game_agility_injury(agility: i32, decreases: i32, rules: Rules) -> i32 {
        if rules == Rules::Bb2020 || rules == Rules::Bb2025 {
            agility + decreases
        } else {
            agility - decreases
        }
    }

    /// Whether a stat value can be further reduced by an in-game injury.
    /// BB2016: only if fewer than 2 injuries already applied.
    /// BB2020/BB2025: always reducible.
    pub fn stat_can_be_reduced_by_injury(
        original_value: i32,
        current_value: i32,
        rules: Rules,
    ) -> bool {
        if rules == Rules::Bb2016 {
            (original_value - current_value) < 2
        } else {
            true
        }
    }
}

impl Default for StatCalc {
    fn default() -> Self {
        Self::new()
    }
}

// Tests mirror ffb-server/src/test/java/com/fumbbl/ffb/server/util/StatCalcTest.java 1:1.
// Java @ParameterizedTest over {BB2020, BB2025} ("modern") becomes a loop over both rulesets.
#[cfg(test)]
mod tests {
    use super::*;

    const MODERN: [Rules; 2] = [Rules::Bb2020, Rules::Bb2025];

    // ── stat_min ─────────────────────────────────────────────────────────────

    #[test]
    fn stat_min_bb2016_all_stats_return_1() {
        for &key in PlayerStatKey::all() {
            assert_eq!(
                StatCalc::stat_min(key, Rules::Bb2016),
                1,
                "Expected min=1 for {:?}",
                key
            );
        }
    }

    #[test]
    fn stat_min_modern_av_returns_3() {
        for rules in MODERN {
            assert_eq!(StatCalc::stat_min(PlayerStatKey::Av, rules), 3);
        }
    }

    #[test]
    fn stat_min_modern_non_av_returns_1() {
        for rules in MODERN {
            assert_eq!(StatCalc::stat_min(PlayerStatKey::Ma, rules), 1);
            assert_eq!(StatCalc::stat_min(PlayerStatKey::St, rules), 1);
            assert_eq!(StatCalc::stat_min(PlayerStatKey::Ag, rules), 1);
            assert_eq!(StatCalc::stat_min(PlayerStatKey::Pa, rules), 1);
        }
    }

    // ── stat_max ─────────────────────────────────────────────────────────────

    #[test]
    fn stat_max_bb2016_ma_st_ag_av_return_10() {
        assert_eq!(StatCalc::stat_max(PlayerStatKey::Ma, Rules::Bb2016), 10);
        assert_eq!(StatCalc::stat_max(PlayerStatKey::St, Rules::Bb2016), 10);
        assert_eq!(StatCalc::stat_max(PlayerStatKey::Ag, Rules::Bb2016), 10);
        assert_eq!(StatCalc::stat_max(PlayerStatKey::Av, Rules::Bb2016), 10);
    }

    #[test]
    fn stat_max_modern_ma_returns_9() {
        for rules in MODERN {
            assert_eq!(StatCalc::stat_max(PlayerStatKey::Ma, rules), 9);
        }
    }

    #[test]
    fn stat_max_modern_st_returns_8() {
        for rules in MODERN {
            assert_eq!(StatCalc::stat_max(PlayerStatKey::St, rules), 8);
        }
    }

    #[test]
    fn stat_max_modern_ag_returns_6() {
        for rules in MODERN {
            assert_eq!(StatCalc::stat_max(PlayerStatKey::Ag, rules), 6);
        }
    }

    #[test]
    fn stat_max_modern_pa_returns_6() {
        for rules in MODERN {
            assert_eq!(StatCalc::stat_max(PlayerStatKey::Pa, rules), 6);
        }
    }

    #[test]
    fn stat_max_modern_av_returns_11() {
        for rules in MODERN {
            assert_eq!(StatCalc::stat_max(PlayerStatKey::Av, rules), 11);
        }
    }

    // ── apply_lasting_injury ─────────────────────────────────────────────────

    #[test]
    fn apply_lasting_injury_bb2016_ma_decreases() {
        assert_eq!(
            StatCalc::apply_lasting_injury(6, PlayerStatKey::Ma, Rules::Bb2016),
            5
        );
    }

    #[test]
    fn apply_lasting_injury_bb2016_ag_decreases() {
        assert_eq!(
            StatCalc::apply_lasting_injury(4, PlayerStatKey::Ag, Rules::Bb2016),
            3
        );
    }

    #[test]
    fn apply_lasting_injury_bb2016_ma_floored_at_1() {
        assert_eq!(
            StatCalc::apply_lasting_injury(1, PlayerStatKey::Ma, Rules::Bb2016),
            1
        );
    }

    #[test]
    fn apply_lasting_injury_modern_ag_increases() {
        for rules in MODERN {
            assert_eq!(
                StatCalc::apply_lasting_injury(3, PlayerStatKey::Ag, rules),
                4
            );
        }
    }

    #[test]
    fn apply_lasting_injury_modern_pa_increases() {
        for rules in MODERN {
            assert_eq!(
                StatCalc::apply_lasting_injury(4, PlayerStatKey::Pa, rules),
                5
            );
        }
    }

    #[test]
    fn apply_lasting_injury_modern_ag_capped_at_max() {
        for rules in MODERN {
            assert_eq!(
                StatCalc::apply_lasting_injury(6, PlayerStatKey::Ag, rules),
                6
            );
        }
    }

    #[test]
    fn apply_lasting_injury_modern_ma_decreases() {
        for rules in MODERN {
            assert_eq!(
                StatCalc::apply_lasting_injury(6, PlayerStatKey::Ma, rules),
                5
            );
        }
    }

    #[test]
    fn apply_lasting_injury_modern_av_floored_at_3() {
        for rules in MODERN {
            assert_eq!(
                StatCalc::apply_lasting_injury(3, PlayerStatKey::Av, rules),
                3
            );
        }
    }

    // ── apply_in_game_agility_injury ─────────────────────────────────────────

    #[test]
    fn apply_in_game_agility_injury_bb2016_decreases() {
        assert_eq!(StatCalc::apply_in_game_agility_injury(4, 1, Rules::Bb2016), 3);
        assert_eq!(StatCalc::apply_in_game_agility_injury(4, 2, Rules::Bb2016), 2);
    }

    #[test]
    fn apply_in_game_agility_injury_modern_increases() {
        for rules in MODERN {
            assert_eq!(StatCalc::apply_in_game_agility_injury(4, 1, rules), 5);
            assert_eq!(StatCalc::apply_in_game_agility_injury(4, 2, rules), 6);
        }
    }

    // ── stat_can_be_reduced_by_injury ────────────────────────────────────────

    #[test]
    fn stat_can_be_reduced_bb2016_no_prior_injuries_true() {
        assert!(StatCalc::stat_can_be_reduced_by_injury(4, 4, Rules::Bb2016));
    }

    #[test]
    fn stat_can_be_reduced_bb2016_one_prior_injury_true() {
        assert!(StatCalc::stat_can_be_reduced_by_injury(4, 3, Rules::Bb2016));
    }

    #[test]
    fn stat_can_be_reduced_bb2016_two_prior_injuries_false() {
        assert!(!StatCalc::stat_can_be_reduced_by_injury(4, 2, Rules::Bb2016));
    }

    #[test]
    fn stat_can_be_reduced_modern_always_true() {
        for rules in MODERN {
            assert!(StatCalc::stat_can_be_reduced_by_injury(4, 4, rules));
            assert!(StatCalc::stat_can_be_reduced_by_injury(4, 3, rules));
            assert!(StatCalc::stat_can_be_reduced_by_injury(4, 2, rules));
            assert!(StatCalc::stat_can_be_reduced_by_injury(4, 1, rules));
        }
    }
}
