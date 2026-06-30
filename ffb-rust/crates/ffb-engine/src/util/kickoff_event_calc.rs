// 1:1 translation of com.fumbbl.ffb.server.util.KickoffEventCalc
//
// Pure kickoff-event roll calculations (Cheering Fans, Brilliant Coaching).
// Both kickoff events compare two team totals. In case of a tie, BOTH teams win a reroll.

pub struct KickoffEventCalc;

impl KickoffEventCalc {
    pub fn new() -> Self {
        Self
    }

    /// Cheering Fans total for one team: D6 roll + fame + cheerleaders.
    pub fn cheering_fans_total(die_roll: i32, fame: i32, cheerleaders: i32) -> i32 {
        die_roll + fame + cheerleaders
    }

    /// Brilliant Coaching total for one team: D6 roll + fame + assistant_coaches - (coach_banned ? 1 : 0).
    pub fn brilliant_coaching_total(
        die_roll: i32,
        fame: i32,
        assistant_coaches: i32,
        coach_banned: bool,
    ) -> i32 {
        die_roll + fame + assistant_coaches + if coach_banned { -1 } else { 0 }
    }

    /// Whether a team gains a reroll: true when its total >= the opponent's total.
    /// Both teams gain a reroll in a tie.
    pub fn gains_extra_reroll(own_total: i32, opponent_total: i32) -> bool {
        own_total >= opponent_total
    }
}

impl Default for KickoffEventCalc {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cheering_fans_total_sums_all_three() {
        // roll=3, fame=2, cheerleaders=1 → 6
        assert_eq!(KickoffEventCalc::cheering_fans_total(3, 2, 1), 6);
    }

    #[test]
    fn cheering_fans_total_no_bonus() {
        assert_eq!(KickoffEventCalc::cheering_fans_total(4, 0, 0), 4);
    }

    #[test]
    fn brilliant_coaching_total_no_ban() {
        // roll=5, fame=1, assistants=2, no ban → 8
        assert_eq!(KickoffEventCalc::brilliant_coaching_total(5, 1, 2, false), 8);
    }

    #[test]
    fn brilliant_coaching_total_with_ban() {
        // roll=5, fame=1, assistants=2, banned → 7
        assert_eq!(KickoffEventCalc::brilliant_coaching_total(5, 1, 2, true), 7);
    }

    #[test]
    fn gains_extra_reroll_when_ahead() {
        assert!(KickoffEventCalc::gains_extra_reroll(7, 5));
    }

    #[test]
    fn gains_extra_reroll_on_tie() {
        // Both teams win on a tie
        assert!(KickoffEventCalc::gains_extra_reroll(5, 5));
    }

    #[test]
    fn no_extra_reroll_when_behind() {
        assert!(!KickoffEventCalc::gains_extra_reroll(4, 6));
    }

    #[test]
    fn brilliant_coaching_zero_values() {
        assert_eq!(KickoffEventCalc::brilliant_coaching_total(1, 0, 0, false), 1);
    }
}
