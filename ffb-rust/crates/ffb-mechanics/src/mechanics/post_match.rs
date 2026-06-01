// ── Kickoff event: Cheering Fans / Brilliant Coaching ────────────────────────

/// Cheering Fans total: D6 roll + fame + cheerleaders.
pub fn cheering_fans_total(die_roll: i32, fame: i32, cheerleaders: i32) -> i32 {
    die_roll + fame + cheerleaders
}

/// Brilliant Coaching total: D6 roll + fame + assistant_coaches - (coach_banned ? 1 : 0).
pub fn brilliant_coaching_total(die_roll: i32, fame: i32, assistant_coaches: i32, coach_banned: bool) -> i32 {
    die_roll + fame + assistant_coaches + if coach_banned { -1 } else { 0 }
}

/// Whether a team gains a reroll: true when own_total >= opponent_total.
/// Both teams gain a reroll in a tie.
pub fn gains_extra_reroll(own_total: i32, opponent_total: i32) -> bool {
    own_total >= opponent_total
}

// ── Post-match fan factor and inducements ─────────────────────────────────────

/// Interpret the fan factor roll.
///
/// * `roll_total` — sum of the fan factor dice (e.g. 3D6)
/// * `fan_factor` — the team's fan factor rating
/// * `score_diff` — team score minus opponent score (positive = winning, negative = losing)
///
/// Returns +1 if winning/drawing AND total > fan_factor;
///         -1 if losing/drawing AND total < fan_factor;
///          0 otherwise.
pub fn interpret_fan_factor_roll(roll_total: i32, fan_factor: i32, score_diff: i32) -> i32 {
    if score_diff >= 0 && roll_total > fan_factor { return 1; }
    if score_diff <= 0 && roll_total < fan_factor { return -1; }
    0
}

/// Interpret a Master Chef roll: each die showing 4, 5, or 6 steals a reroll from the opponent.
pub fn interpret_master_chef_roll(dice: &[i32]) -> i32 {
    dice.iter().filter(|&&d| d > 3).count() as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cheering_fans_adds_all_components() {
        assert_eq!(cheering_fans_total(4, 3, 2), 9);
        assert_eq!(cheering_fans_total(4, 3, 0), 7);
    }

    #[test]
    fn brilliant_coaching_banned_coach_minus1() {
        assert_eq!(brilliant_coaching_total(4, 3, 2, false), 9);
        assert_eq!(brilliant_coaching_total(4, 3, 2, true), 8);
    }

    #[test]
    fn brilliant_coaching_no_assistants() {
        assert_eq!(brilliant_coaching_total(4, 3, 0, false), 7);
    }

    #[test]
    fn scenario_cheering_fans_home_wins() {
        let home_total = cheering_fans_total(4, 5, 2);  // 11
        let away_total = cheering_fans_total(3, 4, 1);  // 8
        assert!(gains_extra_reroll(home_total, away_total));
        assert!(!gains_extra_reroll(away_total, home_total));
    }

    #[test]
    fn scenario_brilliant_coaching_tie_both_win() {
        let home_total = brilliant_coaching_total(3, 5, 2, false);  // 10
        let away_total = brilliant_coaching_total(4, 4, 2, false);  // 10
        assert!(gains_extra_reroll(home_total, away_total));
        assert!(gains_extra_reroll(away_total, home_total));
    }

    #[test]
    fn gains_reroll_higher_wins_tie_both_win() {
        assert!(gains_extra_reroll(8, 5));
        assert!(!gains_extra_reroll(5, 8));
        assert!(gains_extra_reroll(7, 7));
    }

    #[test]
    fn winning_roll_beats_ff_returns_plus1() {
        assert_eq!(interpret_fan_factor_roll(10, 8, 1), 1);
    }

    #[test]
    fn winning_roll_equals_ff_returns_0() {
        assert_eq!(interpret_fan_factor_roll(8, 8, 1), 0);
    }

    #[test]
    fn winning_roll_lower_than_ff_returns_0() {
        assert_eq!(interpret_fan_factor_roll(6, 8, 1), 0);
    }

    #[test]
    fn losing_roll_below_ff_returns_minus1() {
        assert_eq!(interpret_fan_factor_roll(6, 8, -1), -1);
    }

    #[test]
    fn losing_roll_higher_than_ff_returns_0() {
        assert_eq!(interpret_fan_factor_roll(10, 8, -1), 0);
    }

    #[test]
    fn draw_roll_beats_ff_returns_plus1() {
        assert_eq!(interpret_fan_factor_roll(10, 8, 0), 1);
    }

    #[test]
    fn draw_roll_below_ff_returns_minus1() {
        assert_eq!(interpret_fan_factor_roll(6, 8, 0), -1);
    }

    #[test]
    fn draw_roll_equals_ff_returns_0() {
        assert_eq!(interpret_fan_factor_roll(8, 8, 0), 0);
    }

    #[test]
    fn master_chef_all_low_steals_nothing() {
        assert_eq!(interpret_master_chef_roll(&[1, 2, 3]), 0);
    }

    #[test]
    fn master_chef_all_high_steals_all() {
        assert_eq!(interpret_master_chef_roll(&[4, 5, 6]), 3);
    }

    #[test]
    fn master_chef_mixed_steals_partial() {
        assert_eq!(interpret_master_chef_roll(&[3, 4, 6]), 2);
    }

    #[test]
    fn master_chef_empty_steals_0() {
        assert_eq!(interpret_master_chef_roll(&[]), 0);
    }

    #[test]
    fn master_chef_single_die_high_steals_1() {
        assert_eq!(interpret_master_chef_roll(&[4]), 1);
    }

    #[test]
    fn master_chef_single_die_low_steals_0() {
        assert_eq!(interpret_master_chef_roll(&[3]), 0);
    }
}
