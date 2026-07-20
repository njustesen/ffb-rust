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

// Tests mirror ffb-server/src/test/java/com/fumbbl/ffb/server/util/PostMatchCalcTest.java 1:1.
#[cfg(test)]
mod tests {
    use super::*;

    // ── interpret_fan_factor_roll ────────────────────────────────────────────

    #[test]
    fn winning_roll_higher_than_ff_returns1() {
        // scoreDiff > 0 (winning), roll beats fan factor
        assert_eq!(PostMatchCalc::interpret_fan_factor_roll(10, 8, 1), 1);
    }

    #[test]
    fn winning_roll_equal_to_ff_returns0() {
        assert_eq!(PostMatchCalc::interpret_fan_factor_roll(8, 8, 1), 0);
    }

    #[test]
    fn winning_roll_lower_than_ff_returns0() {
        // Not losing, so cannot get -1
        assert_eq!(PostMatchCalc::interpret_fan_factor_roll(6, 8, 1), 0);
    }

    #[test]
    fn losing_roll_lower_than_ff_returns_minus1() {
        assert_eq!(PostMatchCalc::interpret_fan_factor_roll(6, 8, -1), -1);
    }

    #[test]
    fn losing_roll_higher_than_ff_returns0() {
        assert_eq!(PostMatchCalc::interpret_fan_factor_roll(10, 8, -1), 0);
    }

    #[test]
    fn draw_roll_higher_than_ff_returns1() {
        // scoreDiff == 0 satisfies both >= 0 conditions
        assert_eq!(PostMatchCalc::interpret_fan_factor_roll(10, 8, 0), 1);
    }

    #[test]
    fn draw_roll_lower_than_ff_returns_minus1() {
        assert_eq!(PostMatchCalc::interpret_fan_factor_roll(6, 8, 0), -1);
    }

    #[test]
    fn draw_roll_equal_to_ff_returns0() {
        assert_eq!(PostMatchCalc::interpret_fan_factor_roll(8, 8, 0), 0);
    }

    // ── interpret_master_chef_roll ───────────────────────────────────────────

    #[test]
    fn all_low_steals_nothing() {
        assert_eq!(PostMatchCalc::interpret_master_chef_roll(&[1, 2, 3]), 0);
    }

    #[test]
    fn all_high_steals_all() {
        assert_eq!(PostMatchCalc::interpret_master_chef_roll(&[4, 5, 6]), 3);
    }

    #[test]
    fn mixed_steals_partial() {
        assert_eq!(PostMatchCalc::interpret_master_chef_roll(&[3, 4, 6]), 2);
    }

    #[test]
    fn single_die_high_steals1() {
        assert_eq!(PostMatchCalc::interpret_master_chef_roll(&[4]), 1);
    }

    #[test]
    fn single_die_low_steals0() {
        assert_eq!(PostMatchCalc::interpret_master_chef_roll(&[3]), 0);
    }

    #[test]
    fn empty_dice_steals0() {
        assert_eq!(PostMatchCalc::interpret_master_chef_roll(&[]), 0);
    }
}
