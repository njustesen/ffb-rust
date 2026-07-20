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

// NOTE: tests duplicating StatCalcTest.java live in the 1:1 mirror module
// (ffb-engine/src/util/stat_calc.rs); redundant copies were removed here.
