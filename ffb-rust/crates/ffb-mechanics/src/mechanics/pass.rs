use ffb_model::enums::{PassingDistance, Rules};
use crate::modifiers::Modifier;

/// Result of a pass attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PassResult {
    Accurate,
    Inaccurate,
    WildlyInaccurate,
    Fumble,
    SavedFumble,
}

/// Distance modifier applied to the pass target number (BB2016 / agility-based).
///
/// Quick Pass  +1, Short Pass 0, Long Pass -1, Long Bomb -2
pub fn distance_modifier_bb2016(distance: PassingDistance) -> i32 {
    match distance {
        PassingDistance::QuickPass => 1,
        PassingDistance::ShortPass => 0,
        PassingDistance::LongPass => -1,
        PassingDistance::LongBomb => -2,
        PassingDistance::PassToPartner => 0,
    }
}

/// Distance modifier added to the PA roll (BB2020/2025).
///
/// Quick Pass 0, Short Pass +1, Long Pass +2, Long Bomb +3
pub fn distance_modifier_bb2020(distance: PassingDistance) -> i32 {
    match distance {
        PassingDistance::QuickPass => 0,
        PassingDistance::ShortPass => 1,
        PassingDistance::LongPass => 2,
        PassingDistance::LongBomb => 3,
        PassingDistance::PassToPartner => 0,
    }
}

/// Whether a BB2016 pass roll is a modified fumble (not counting the natural-1 rule).
///
/// A modified fumble occurs when: `roll + dist_mod - modifier_sum ≤ 1`
/// Natural 1 is a direct fumble handled separately.
pub fn is_modified_fumble_bb2016(roll: i32, distance: PassingDistance, modifier_sum: i32) -> bool {
    let dist_mod = distance_modifier_bb2016(distance);
    roll + dist_mod - modifier_sum <= 1
}

/// Minimum roll to complete a pass in BB2016 (agility-based, like dodge).
///
/// Matches Java PassMechanic.minimumRollInternal:
///   max(max(2 - (dist_mod - mods), 2), 7 - min(ag, 6) - dist_mod + mods)
///
/// The first inner max ensures the target is always above the fumble boundary
/// (a long bomb cannot have a target below 4 even with superhuman AG).
pub fn minimum_roll_pass_bb2016(agility: i32, distance: PassingDistance, modifiers: &[Modifier]) -> i32 {
    let modifier_sum: i32 = modifiers.iter().map(|m| m.value).sum();
    let dist_mod = distance_modifier_bb2016(distance);
    let ag_capped = agility.min(6);
    let ag_based = 7 - ag_capped - dist_mod + modifier_sum;
    let fumble_boundary = 2 - dist_mod + modifier_sum; // ensures target > fumble line
    ag_based.max(fumble_boundary).max(2)
}

/// Minimum roll to complete a pass in BB2020/BB2025 (PA-based).
///
/// Returns `None` when the player has no passing ability (PA = 0 means cannot pass).
/// Formula: max(2, pa + distance_modifier + sum_of_modifiers)
pub fn minimum_roll_pass_bb2020(passing_ability: i32, distance: PassingDistance, modifiers: &[Modifier]) -> Option<i32> {
    if passing_ability <= 0 {
        return None;
    }
    let modifier_sum: i32 = modifiers.iter().map(|m| m.value).sum();
    let dist_mod = distance_modifier_bb2020(distance);
    Some((passing_ability + dist_mod + modifier_sum).max(2))
}

/// Evaluate a BB2016 pass roll against the computed minimum.
pub fn evaluate_pass_bb2016(
    agility: i32,
    roll: i32,
    distance: PassingDistance,
    modifiers: &[Modifier],
    has_sure_hands: bool,
) -> PassResult {
    let modifier_sum: i32 = modifiers.iter().map(|m| m.value).sum();
    let dist_mod = distance_modifier_bb2016(distance);

    if roll == 6 {
        return PassResult::Accurate;
    }
    if roll == 1 {
        return PassResult::Fumble;
    }
    // Modified fumble: roll + dist_mod - modifiers ≤ 1
    if roll + dist_mod - modifier_sum <= 1 {
        if has_sure_hands {
            return PassResult::SavedFumble;
        }
        return PassResult::Fumble;
    }
    let minimum = minimum_roll_pass_bb2016(agility, distance, modifiers);
    if roll < minimum {
        PassResult::Inaccurate
    } else {
        PassResult::Accurate
    }
}

/// Evaluate a BB2020/BB2025 pass roll.
///
/// Returns `Fumble` when the player has no passing ability or rolls a natural 1.
pub fn evaluate_pass_bb2020(
    passing_ability: i32,
    roll: i32,
    distance: PassingDistance,
    modifiers: &[Modifier],
    has_sure_hands: bool,
) -> PassResult {
    if passing_ability <= 0 || roll == 1 {
        if has_sure_hands {
            return PassResult::SavedFumble;
        }
        return PassResult::Fumble;
    }

    let modifier_sum: i32 = modifiers.iter().map(|m| m.value).sum();
    let dist_mod = distance_modifier_bb2020(distance);
    // result_after_modifiers = roll - modifier_sum - dist_mod compared to PA
    let effective = roll - modifier_sum - dist_mod;

    if roll == 6 || effective >= passing_ability {
        PassResult::Accurate
    } else if effective <= 1 {
        PassResult::WildlyInaccurate
    } else {
        PassResult::Inaccurate
    }
}

/// Dispatch pass evaluation by rules edition.
pub fn evaluate_pass(
    rules: Rules,
    stat: i32,
    roll: i32,
    distance: PassingDistance,
    modifiers: &[Modifier],
    has_sure_hands: bool,
) -> PassResult {
    match rules {
        Rules::Bb2016 => evaluate_pass_bb2016(stat, roll, distance, modifiers, has_sure_hands),
        Rules::Bb2025 | Rules::Common => {
            // BB2025: "wildly inaccurate" was removed — effective <= 1 is now a Fumble.
            // Java bb2025/PassMechanic.evaluatePass: `resultAfterModifiers <= 1 → FUMBLE`
            let result = evaluate_pass_bb2020(stat, roll, distance, modifiers, has_sure_hands);
            if result == PassResult::WildlyInaccurate { PassResult::Fumble } else { result }
        }
        Rules::Bb2020 => evaluate_pass_bb2020(stat, roll, distance, modifiers, has_sure_hands),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bb2016_minimum_short_pass_ag3() {
        // AG3, short pass (mod 0): 7 - 3 - 0 = 4
        assert_eq!(minimum_roll_pass_bb2016(3, PassingDistance::ShortPass, &[]), 4);
    }

    #[test]
    fn bb2016_minimum_long_pass_ag4() {
        // AG4, long pass (mod -1): 7 - 4 - (-1) = 4 → but wait: 7 - 4 + 1 = 4
        assert_eq!(minimum_roll_pass_bb2016(4, PassingDistance::LongPass, &[]), 4);
    }

    #[test]
    fn bb2016_minimum_capped_at_2() {
        // AG6, quick pass (mod +1): 7 - 6 - 1 = 0 → capped at 2
        assert_eq!(minimum_roll_pass_bb2016(6, PassingDistance::QuickPass, &[]), 2);
    }

    #[test]
    fn bb2016_long_bomb_minimum_high_ag() {
        // AG6, long bomb (mod -2): fumble boundary = 2 - (-2) = 4; ag-based = 7-6+2 = 3
        // max(3, 4, 2) = 4
        assert_eq!(minimum_roll_pass_bb2016(6, PassingDistance::LongBomb, &[]), 4);
    }

    #[test]
    fn bb2016_short_pass_ag6() {
        // AG6, short pass (mod 0): ag-based = 7-6-0 = 1; fumble = 2-0 = 2; max = 2
        assert_eq!(minimum_roll_pass_bb2016(6, PassingDistance::ShortPass, &[]), 2);
    }

    #[test]
    fn bb2020_no_pa_returns_none() {
        assert_eq!(minimum_roll_pass_bb2020(0, PassingDistance::ShortPass, &[]), None);
    }

    #[test]
    fn bb2020_pa2_short_pass() {
        // PA2 + dist_mod 1 = 3
        assert_eq!(minimum_roll_pass_bb2020(2, PassingDistance::ShortPass, &[]), Some(3));
    }

    #[test]
    fn bb2020_natural_one_is_fumble() {
        assert_eq!(
            evaluate_pass_bb2020(3, 1, PassingDistance::ShortPass, &[], false),
            PassResult::Fumble
        );
    }

    #[test]
    fn bb2020_natural_six_is_accurate() {
        assert_eq!(
            evaluate_pass_bb2020(3, 6, PassingDistance::LongBomb, &[], false),
            PassResult::Accurate
        );
    }

    #[test]
    fn bb2016_natural_six_is_accurate() {
        assert_eq!(
            evaluate_pass_bb2016(1, 6, PassingDistance::LongBomb, &[], false),
            PassResult::Accurate
        );
    }

    #[test]
    fn bb2016_natural_one_is_fumble() {
        assert_eq!(
            evaluate_pass_bb2016(6, 1, PassingDistance::QuickPass, &[], false),
            PassResult::Fumble
        );
    }

    #[test]
    fn bb2016_long_bomb_ag4() {
        // AG4, long bomb: ag_based = 7-4+2 = 5; fumble_boundary = 4; max = 5
        assert_eq!(minimum_roll_pass_bb2016(4, PassingDistance::LongBomb, &[]), 5);
    }

    #[test]
    fn bb2016_with_positive_modifier_harder() {
        // AG4, short pass, +1 modifier: 7-4-0+1 = 4; fumble_boundary=2+1=3; max=4
        assert_eq!(
            minimum_roll_pass_bb2016(4, PassingDistance::ShortPass, &[Modifier::new("test", 1, Rules::Common)]),
            4
        );
    }

    #[test]
    fn bb2016_with_negative_modifier_easier_floor_at_2() {
        // AG4, short pass, -2 modifier: 7-4-0-2 = 1 → 2; fumble=2-0-2=0 → max = 2
        assert_eq!(
            minimum_roll_pass_bb2016(4, PassingDistance::ShortPass, &[Modifier::new("test", -2, Rules::Common)]),
            2
        );
    }

    #[test]
    fn bb2016_ag4_all_distances() {
        // QUICK_PASS: 7-4-1=2, fumble=2-1=1; max=2
        assert_eq!(minimum_roll_pass_bb2016(4, PassingDistance::QuickPass, &[]), 2);
        // SHORT_PASS: 7-4-0=3, fumble=2; max=3
        assert_eq!(minimum_roll_pass_bb2016(4, PassingDistance::ShortPass, &[]), 3);
        // LONG_PASS: 7-4+1=4, fumble=3; max=4
        assert_eq!(minimum_roll_pass_bb2016(4, PassingDistance::LongPass, &[]), 4);
        // LONG_BOMB: 7-4+2=5, fumble=4; max=5
        assert_eq!(minimum_roll_pass_bb2016(4, PassingDistance::LongBomb, &[]), 5);
    }

    #[test]
    fn bb2020_pa3_quick_pass() {
        // PA3 + dist_mod 0 = 3
        assert_eq!(minimum_roll_pass_bb2020(3, PassingDistance::QuickPass, &[]), Some(3));
    }

    #[test]
    fn bb2020_pa3_long_bomb() {
        // PA3 + dist_mod 3 = 6
        assert_eq!(minimum_roll_pass_bb2020(3, PassingDistance::LongBomb, &[]), Some(6));
    }

    #[test]
    fn bb2020_with_modifier() {
        // PA3, short pass (1), +1 modifier = 5
        assert_eq!(
            minimum_roll_pass_bb2020(3, PassingDistance::ShortPass, &[Modifier::new("test", 1, Rules::Common)]),
            Some(5)
        );
    }

    #[test]
    fn bb2020_floor_at_2() {
        // PA2 + dist_mod 0 - 5 modifiers = -3 → 2
        assert_eq!(
            minimum_roll_pass_bb2020(2, PassingDistance::QuickPass, &[Modifier::new("test", -5, Rules::Common)]),
            Some(2)
        );
    }

    #[test]
    fn bb2016_natural_one_quick_pass_not_modified_fumble() {
        // roll=1, quick pass dist_mod=+1: 1+1-0=2 > 1 → NOT a modified fumble
        assert!(!is_modified_fumble_bb2016(1, PassingDistance::QuickPass, 0));
    }

    #[test]
    fn bb2016_long_bomb_roll2_is_fumble() {
        // roll 2 + dist_mod(-2) - 0 = 0 <= 1 → fumble
        assert!(is_modified_fumble_bb2016(2, PassingDistance::LongBomb, 0));
    }

    #[test]
    fn bb2016_short_pass_roll2_not_fumble() {
        // roll 2 + dist_mod(0) - 0 = 2 > 1 → not fumble
        assert!(!is_modified_fumble_bb2016(2, PassingDistance::ShortPass, 0));
    }

    #[test]
    fn bb2016_quick_pass_roll1_not_modified_fumble() {
        // roll 1 + dist_mod(+1) - 0 = 2 > 1 → not a modified fumble
        assert!(!is_modified_fumble_bb2016(1, PassingDistance::QuickPass, 0));
    }

    // ── BB2025 pass evaluation ────────────────────────────────────────────────

    #[test]
    fn bb2025_effective_le_1_is_fumble_not_wildly_inaccurate() {
        // PA=4, roll=2, short pass: effective = 2 - 1 = 1 ≤ 1 → Fumble in BB2025 (not WildlyInaccurate)
        let result = evaluate_pass(Rules::Bb2025, 4, 2, PassingDistance::ShortPass, &[], false);
        assert_eq!(result, PassResult::Fumble);
    }

    #[test]
    fn bb2020_effective_le_1_is_wildly_inaccurate() {
        // Same scenario in BB2020 → WildlyInaccurate
        let result = evaluate_pass(Rules::Bb2020, 4, 2, PassingDistance::ShortPass, &[], false);
        assert_eq!(result, PassResult::WildlyInaccurate);
    }

    #[test]
    fn bb2025_natural_1_is_fumble() {
        let result = evaluate_pass(Rules::Bb2025, 3, 1, PassingDistance::QuickPass, &[], false);
        assert_eq!(result, PassResult::Fumble);
    }

    #[test]
    fn bb2025_accurate_pass_unchanged() {
        // PA=3, roll=6 → always Accurate
        let result = evaluate_pass(Rules::Bb2025, 3, 6, PassingDistance::LongPass, &[], false);
        assert_eq!(result, PassResult::Accurate);
    }

    #[test]
    fn bb2025_normal_inaccurate_unchanged() {
        // PA=4, roll=3, short pass: effective = 3 - 1 = 2, not ≥ 4, not ≤ 1 → Inaccurate
        let result = evaluate_pass(Rules::Bb2025, 4, 3, PassingDistance::ShortPass, &[], false);
        assert_eq!(result, PassResult::Inaccurate);
    }
}
