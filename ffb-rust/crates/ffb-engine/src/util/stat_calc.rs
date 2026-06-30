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

#[cfg(test)]
mod tests {
    use super::*;

    // ── stat_min ─────────────────────────────────────────────────────────────

    #[test]
    fn stat_min_bb2016_all_1() {
        for &key in PlayerStatKey::all() {
            assert_eq!(StatCalc::stat_min(key, Rules::Bb2016), 1, "{:?}", key);
        }
    }

    #[test]
    fn stat_min_bb2020_av_is_3() {
        assert_eq!(StatCalc::stat_min(PlayerStatKey::Av, Rules::Bb2020), 3);
    }

    #[test]
    fn stat_min_bb2025_av_is_3() {
        assert_eq!(StatCalc::stat_min(PlayerStatKey::Av, Rules::Bb2025), 3);
    }

    #[test]
    fn stat_min_bb2020_non_av_is_1() {
        assert_eq!(StatCalc::stat_min(PlayerStatKey::Ma, Rules::Bb2020), 1);
        assert_eq!(StatCalc::stat_min(PlayerStatKey::St, Rules::Bb2020), 1);
        assert_eq!(StatCalc::stat_min(PlayerStatKey::Ag, Rules::Bb2020), 1);
    }

    // ── stat_max ─────────────────────────────────────────────────────────────

    #[test]
    fn stat_max_bb2016_common_stats_are_10() {
        assert_eq!(StatCalc::stat_max(PlayerStatKey::Ma, Rules::Bb2016), 10);
        assert_eq!(StatCalc::stat_max(PlayerStatKey::St, Rules::Bb2016), 10);
        assert_eq!(StatCalc::stat_max(PlayerStatKey::Ag, Rules::Bb2016), 10);
        assert_eq!(StatCalc::stat_max(PlayerStatKey::Av, Rules::Bb2016), 10);
    }

    #[test]
    fn stat_max_bb2020_specific_caps() {
        assert_eq!(StatCalc::stat_max(PlayerStatKey::Ma, Rules::Bb2020), 9);
        assert_eq!(StatCalc::stat_max(PlayerStatKey::St, Rules::Bb2020), 8);
        assert_eq!(StatCalc::stat_max(PlayerStatKey::Ag, Rules::Bb2020), 6);
        assert_eq!(StatCalc::stat_max(PlayerStatKey::Pa, Rules::Bb2020), 6);
        assert_eq!(StatCalc::stat_max(PlayerStatKey::Av, Rules::Bb2020), 11);
    }

    #[test]
    fn stat_max_bb2025_same_as_bb2020() {
        for &key in PlayerStatKey::all() {
            assert_eq!(
                StatCalc::stat_max(key, Rules::Bb2025),
                StatCalc::stat_max(key, Rules::Bb2020),
                "{:?}",
                key
            );
        }
    }

    // ── apply_lasting_injury ─────────────────────────────────────────────────

    #[test]
    fn apply_lasting_injury_bb2016_decreases_ma() {
        assert_eq!(
            StatCalc::apply_lasting_injury(6, PlayerStatKey::Ma, Rules::Bb2016),
            5
        );
    }

    #[test]
    fn apply_lasting_injury_bb2016_floored_at_1() {
        assert_eq!(
            StatCalc::apply_lasting_injury(1, PlayerStatKey::St, Rules::Bb2016),
            1
        );
    }

    #[test]
    fn apply_lasting_injury_bb2020_ag_increases() {
        // BB2020 AG worsens (higher = worse target)
        assert_eq!(
            StatCalc::apply_lasting_injury(3, PlayerStatKey::Ag, Rules::Bb2020),
            4
        );
    }

    #[test]
    fn apply_lasting_injury_bb2020_ag_capped_at_max() {
        assert_eq!(
            StatCalc::apply_lasting_injury(6, PlayerStatKey::Ag, Rules::Bb2020),
            6
        );
    }

    #[test]
    fn apply_lasting_injury_bb2020_ma_decreases() {
        assert_eq!(
            StatCalc::apply_lasting_injury(7, PlayerStatKey::Ma, Rules::Bb2020),
            6
        );
    }

    #[test]
    fn apply_lasting_injury_bb2020_av_floored_at_3() {
        assert_eq!(
            StatCalc::apply_lasting_injury(3, PlayerStatKey::Av, Rules::Bb2020),
            3
        );
    }

    // ── apply_in_game_agility_injury ─────────────────────────────────────────

    #[test]
    fn in_game_agility_injury_bb2016_decreases() {
        assert_eq!(StatCalc::apply_in_game_agility_injury(4, 1, Rules::Bb2016), 3);
    }

    #[test]
    fn in_game_agility_injury_bb2020_increases() {
        assert_eq!(StatCalc::apply_in_game_agility_injury(3, 1, Rules::Bb2020), 4);
    }

    // ── stat_can_be_reduced_by_injury ────────────────────────────────────────

    #[test]
    fn stat_reducible_bb2016_zero_injuries() {
        assert!(StatCalc::stat_can_be_reduced_by_injury(4, 4, Rules::Bb2016));
    }

    #[test]
    fn stat_reducible_bb2016_one_injury() {
        assert!(StatCalc::stat_can_be_reduced_by_injury(4, 3, Rules::Bb2016));
    }

    #[test]
    fn stat_not_reducible_bb2016_two_injuries() {
        assert!(!StatCalc::stat_can_be_reduced_by_injury(4, 2, Rules::Bb2016));
    }

    #[test]
    fn stat_always_reducible_bb2020() {
        assert!(StatCalc::stat_can_be_reduced_by_injury(4, 1, Rules::Bb2020));
        assert!(StatCalc::stat_can_be_reduced_by_injury(4, 2, Rules::Bb2020));
    }
}
