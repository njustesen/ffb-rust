use ffb_model::enums::{PlayerStatKey, Rules};

pub fn stat_min(key: PlayerStatKey, rules: Rules) -> i32 {
    if rules == Rules::Bb2020 || rules == Rules::Bb2025 {
        if key == PlayerStatKey::Av { 3 } else { 1 }
    } else {
        1
    }
}

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
        match key {
            PlayerStatKey::Ma | PlayerStatKey::St | PlayerStatKey::Ag | PlayerStatKey::Av => 10,
            PlayerStatKey::Pa => 0,
        }
    }
}

pub fn apply_lasting_injury(value: i32, key: PlayerStatKey, rules: Rules) -> i32 {
    let min = stat_min(key, rules);
    let max = stat_max(key, rules);
    if rules == Rules::Bb2020 || rules == Rules::Bb2025 {
        if key == PlayerStatKey::Ag || key == PlayerStatKey::Pa {
            return (value + 1).min(max);
        }
        (value - 1).max(min)
    } else {
        (value - 1).max(min)
    }
}

pub fn apply_in_game_agility_injury(agility: i32, decreases: i32, rules: Rules) -> i32 {
    if rules == Rules::Bb2020 || rules == Rules::Bb2025 {
        agility + decreases
    } else {
        agility - decreases
    }
}

pub fn stat_can_be_reduced_by_injury(original_value: i32, current_value: i32, rules: Rules) -> bool {
    if rules == Rules::Bb2016 {
        (original_value - current_value) < 2
    } else {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── stat_min ──────────────────────────────────────────────────────────────

    #[test]
    fn stat_min_bb2016_all_return_1() {
        for &key in PlayerStatKey::all() {
            assert_eq!(1, stat_min(key, Rules::Bb2016), "Expected 1 for {:?}", key);
        }
    }

    #[test]
    fn stat_min_modern_av_returns_3() {
        assert_eq!(3, stat_min(PlayerStatKey::Av, Rules::Bb2020));
        assert_eq!(3, stat_min(PlayerStatKey::Av, Rules::Bb2025));
    }

    #[test]
    fn stat_min_modern_non_av_returns_1() {
        for rules in [Rules::Bb2020, Rules::Bb2025] {
            assert_eq!(1, stat_min(PlayerStatKey::Ma, rules));
            assert_eq!(1, stat_min(PlayerStatKey::St, rules));
            assert_eq!(1, stat_min(PlayerStatKey::Ag, rules));
            assert_eq!(1, stat_min(PlayerStatKey::Pa, rules));
        }
    }

    // ── stat_max ──────────────────────────────────────────────────────────────

    #[test]
    fn stat_max_bb2016_ma_st_ag_av_return_10() {
        assert_eq!(10, stat_max(PlayerStatKey::Ma, Rules::Bb2016));
        assert_eq!(10, stat_max(PlayerStatKey::St, Rules::Bb2016));
        assert_eq!(10, stat_max(PlayerStatKey::Ag, Rules::Bb2016));
        assert_eq!(10, stat_max(PlayerStatKey::Av, Rules::Bb2016));
    }

    #[test]
    fn stat_max_modern_per_stat() {
        for rules in [Rules::Bb2020, Rules::Bb2025] {
            assert_eq!(9, stat_max(PlayerStatKey::Ma, rules));
            assert_eq!(8, stat_max(PlayerStatKey::St, rules));
            assert_eq!(6, stat_max(PlayerStatKey::Ag, rules));
            assert_eq!(6, stat_max(PlayerStatKey::Pa, rules));
            assert_eq!(11, stat_max(PlayerStatKey::Av, rules));
        }
    }

    // ── apply_lasting_injury ──────────────────────────────────────────────────

    #[test]
    fn lasting_injury_bb2016_ma_decreases() {
        assert_eq!(5, apply_lasting_injury(6, PlayerStatKey::Ma, Rules::Bb2016));
    }

    #[test]
    fn lasting_injury_bb2016_ag_decreases() {
        assert_eq!(3, apply_lasting_injury(4, PlayerStatKey::Ag, Rules::Bb2016));
    }

    #[test]
    fn lasting_injury_bb2016_floored_at_min() {
        assert_eq!(1, apply_lasting_injury(1, PlayerStatKey::Ma, Rules::Bb2016));
    }

    #[test]
    fn lasting_injury_modern_ag_increases() {
        for rules in [Rules::Bb2020, Rules::Bb2025] {
            assert_eq!(4, apply_lasting_injury(3, PlayerStatKey::Ag, rules));
        }
    }

    #[test]
    fn lasting_injury_modern_pa_increases() {
        for rules in [Rules::Bb2020, Rules::Bb2025] {
            assert_eq!(5, apply_lasting_injury(4, PlayerStatKey::Pa, rules));
        }
    }

    #[test]
    fn lasting_injury_modern_ag_capped_at_6() {
        for rules in [Rules::Bb2020, Rules::Bb2025] {
            assert_eq!(6, apply_lasting_injury(6, PlayerStatKey::Ag, rules));
        }
    }

    #[test]
    fn lasting_injury_modern_ma_decreases() {
        for rules in [Rules::Bb2020, Rules::Bb2025] {
            assert_eq!(5, apply_lasting_injury(6, PlayerStatKey::Ma, rules));
        }
    }

    #[test]
    fn lasting_injury_modern_av_floored_at_3() {
        for rules in [Rules::Bb2020, Rules::Bb2025] {
            assert_eq!(3, apply_lasting_injury(3, PlayerStatKey::Av, rules));
        }
    }

    // ── apply_in_game_agility_injury ──────────────────────────────────────────

    #[test]
    fn in_game_agility_bb2016_decreases() {
        assert_eq!(3, apply_in_game_agility_injury(4, 1, Rules::Bb2016));
        assert_eq!(2, apply_in_game_agility_injury(4, 2, Rules::Bb2016));
    }

    #[test]
    fn in_game_agility_modern_increases() {
        for rules in [Rules::Bb2020, Rules::Bb2025] {
            assert_eq!(5, apply_in_game_agility_injury(4, 1, rules));
            assert_eq!(6, apply_in_game_agility_injury(4, 2, rules));
        }
    }

    // ── stat_can_be_reduced_by_injury ─────────────────────────────────────────

    #[test]
    fn reducible_bb2016_no_prior_true() {
        assert!(stat_can_be_reduced_by_injury(4, 4, Rules::Bb2016));
    }

    #[test]
    fn reducible_bb2016_one_prior_true() {
        assert!(stat_can_be_reduced_by_injury(4, 3, Rules::Bb2016));
    }

    #[test]
    fn reducible_bb2016_two_prior_false() {
        assert!(!stat_can_be_reduced_by_injury(4, 2, Rules::Bb2016));
    }

    #[test]
    fn reducible_modern_always_true() {
        for rules in [Rules::Bb2020, Rules::Bb2025] {
            assert!(stat_can_be_reduced_by_injury(4, 4, rules));
            assert!(stat_can_be_reduced_by_injury(4, 3, rules));
            assert!(stat_can_be_reduced_by_injury(4, 2, rules));
            assert!(stat_can_be_reduced_by_injury(4, 1, rules));
        }
    }
}
