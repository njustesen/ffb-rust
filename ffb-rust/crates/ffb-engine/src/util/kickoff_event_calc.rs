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

// Tests mirror ffb-java/ffb-server/src/test/java/com/fumbbl/ffb/server/util/KickoffEventCalcTest.java 1:1
#[cfg(test)]
mod tests {
    use super::*;

    // ── cheering_fans_total ───────────────────────────────────────────────────

    #[test]
    fn cheering_fans_roll4_fame3_cheerleaders2_is9() {
        assert_eq!(9, KickoffEventCalc::cheering_fans_total(4, 3, 2));
    }

    #[test]
    fn cheering_fans_no_cheerleaders() {
        assert_eq!(7, KickoffEventCalc::cheering_fans_total(4, 3, 0));
    }

    // ── brilliant_coaching_total ──────────────────────────────────────────────

    #[test]
    fn brilliant_coaching_normal_coach() {
        assert_eq!(9, KickoffEventCalc::brilliant_coaching_total(4, 3, 2, false));
    }

    #[test]
    fn brilliant_coaching_banned_coach_minus1() {
        assert_eq!(8, KickoffEventCalc::brilliant_coaching_total(4, 3, 2, true));
    }

    #[test]
    fn brilliant_coaching_no_assistants() {
        assert_eq!(7, KickoffEventCalc::brilliant_coaching_total(4, 3, 0, false));
    }

    // ── gains_extra_reroll ────────────────────────────────────────────────────

    #[test]
    fn home_wins_higher_total() {
        assert!(KickoffEventCalc::gains_extra_reroll(8, 5));
    }

    #[test]
    fn away_wins_home_does_not() {
        assert!(!KickoffEventCalc::gains_extra_reroll(5, 8));
    }

    #[test]
    fn tie_both_gain_reroll() {
        assert!(KickoffEventCalc::gains_extra_reroll(7, 7));
    }

    // ── combined scenario ─────────────────────────────────────────────────────

    #[test]
    fn scenario_cheering_fans_home_wins() {
        let home_total = KickoffEventCalc::cheering_fans_total(4, 5, 2); // 11
        let away_total = KickoffEventCalc::cheering_fans_total(3, 4, 1); // 8
        assert!(KickoffEventCalc::gains_extra_reroll(home_total, away_total));
        assert!(!KickoffEventCalc::gains_extra_reroll(away_total, home_total));
    }

    #[test]
    fn scenario_brilliant_coaching_tie_both_win() {
        let home_total = KickoffEventCalc::brilliant_coaching_total(3, 5, 2, false); // 10
        let away_total = KickoffEventCalc::brilliant_coaching_total(4, 4, 2, false); // 10
        assert!(KickoffEventCalc::gains_extra_reroll(home_total, away_total));
        assert!(KickoffEventCalc::gains_extra_reroll(away_total, home_total));
    }
}
