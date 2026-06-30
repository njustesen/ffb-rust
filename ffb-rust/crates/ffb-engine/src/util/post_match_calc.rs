// 1:1 translation of com.fumbbl.ffb.server.util.PostMatchCalc
//
// Pure post-match and kickoff-event roll calculations extracted from DiceInterpreter.

pub struct PostMatchCalc;

impl PostMatchCalc {
    pub fn new() -> Self {
        Self
    }

    /// Interpret the fan factor roll result at the end of a game.
    ///
    /// `roll_total`  — Sum of the fan factor dice (usually 3D6).
    /// `fan_factor`  — The team's fan factor rating.
    /// `score_diff`  — (team score - opponent score): positive = winning, negative = losing, 0 = draw.
    ///
    /// Returns +1 if winning/drawing AND roll_total > fan_factor;
    ///         -1 if losing/drawing AND roll_total < fan_factor;
    ///          0 otherwise.
    pub fn interpret_fan_factor_roll(roll_total: i32, fan_factor: i32, score_diff: i32) -> i32 {
        if score_diff >= 0 && roll_total > fan_factor {
            return 1;
        }
        if score_diff <= 0 && roll_total < fan_factor {
            return -1;
        }
        0
    }

    /// Interpret a Master Chef roll: each die that shows 4, 5, or 6 steals a reroll from the opponent.
    ///
    /// `dice` — individual die results (usually 3 dice).
    /// Returns number of rerolls stolen (0 to dice.len()).
    pub fn interpret_master_chef_roll(dice: &[i32]) -> i32 {
        dice.iter().filter(|&&d| d > 3).count() as i32
    }
}

impl Default for PostMatchCalc {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── interpret_fan_factor_roll ────────────────────────────────────────────

    #[test]
    fn fan_factor_winning_roll_beats_factor_gains_fan() {
        // winning (score_diff > 0), roll > fan_factor → +1
        assert_eq!(PostMatchCalc::interpret_fan_factor_roll(10, 8, 1), 1);
    }

    #[test]
    fn fan_factor_draw_roll_beats_factor_gains_fan() {
        // draw (score_diff == 0), roll > fan_factor → +1
        assert_eq!(PostMatchCalc::interpret_fan_factor_roll(9, 7, 0), 1);
    }

    #[test]
    fn fan_factor_losing_roll_below_factor_loses_fan() {
        // losing (score_diff < 0), roll < fan_factor → -1
        assert_eq!(PostMatchCalc::interpret_fan_factor_roll(5, 8, -2), -1);
    }

    #[test]
    fn fan_factor_draw_roll_below_factor_loses_fan() {
        // draw (score_diff == 0), roll < fan_factor → -1
        assert_eq!(PostMatchCalc::interpret_fan_factor_roll(6, 8, 0), -1);
    }

    #[test]
    fn fan_factor_roll_equals_factor_no_change() {
        assert_eq!(PostMatchCalc::interpret_fan_factor_roll(8, 8, 1), 0);
    }

    #[test]
    fn fan_factor_winning_roll_below_factor_no_change() {
        // winning but roll ≤ fan_factor → 0
        assert_eq!(PostMatchCalc::interpret_fan_factor_roll(7, 9, 2), 0);
    }

    // ── interpret_master_chef_roll ───────────────────────────────────────────

    #[test]
    fn master_chef_all_high_steals_three() {
        assert_eq!(PostMatchCalc::interpret_master_chef_roll(&[4, 5, 6]), 3);
    }

    #[test]
    fn master_chef_all_low_steals_none() {
        assert_eq!(PostMatchCalc::interpret_master_chef_roll(&[1, 2, 3]), 0);
    }

    #[test]
    fn master_chef_mixed_steals_two() {
        assert_eq!(PostMatchCalc::interpret_master_chef_roll(&[1, 4, 6]), 2);
    }

    #[test]
    fn master_chef_empty_slice_steals_none() {
        assert_eq!(PostMatchCalc::interpret_master_chef_roll(&[]), 0);
    }

    #[test]
    fn master_chef_exactly_four_is_stolen() {
        // boundary: 4 is strictly > 3
        assert_eq!(PostMatchCalc::interpret_master_chef_roll(&[4]), 1);
    }

    #[test]
    fn master_chef_exactly_three_not_stolen() {
        // boundary: 3 is not > 3
        assert_eq!(PostMatchCalc::interpret_master_chef_roll(&[3]), 0);
    }
}
