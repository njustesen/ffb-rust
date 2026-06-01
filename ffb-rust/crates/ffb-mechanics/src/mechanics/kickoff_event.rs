/// Cheering Fans total for one team: D6 roll + fame + cheerleaders.
///
/// Mirrors Java `KickoffEventCalc.cheeringFansTotal()`.
pub fn cheering_fans_total(die_roll: i32, fame: i32, cheerleaders: i32) -> i32 {
    die_roll + fame + cheerleaders
}

/// Brilliant Coaching total for one team: D6 roll + fame + assistant_coaches - (banned ? 1 : 0).
///
/// Mirrors Java `KickoffEventCalc.brilliantCoachingTotal()`.
pub fn brilliant_coaching_total(die_roll: i32, fame: i32, assistant_coaches: i32, coach_banned: bool) -> i32 {
    die_roll + fame + assistant_coaches + if coach_banned { -1 } else { 0 }
}

/// Whether a team gains an extra reroll: true when its total >= the opponent's total.
///
/// Both teams gain a reroll on a tie (the caller checks both directions).
pub fn gains_extra_reroll(own_total: i32, opponent_total: i32) -> bool {
    own_total >= opponent_total
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── cheering_fans_total ──────────────────────────────────────────────────

    #[test]
    fn cheering_fans_roll4_fame3_cheerleaders2_is9() {
        assert_eq!(cheering_fans_total(4, 3, 2), 9);
    }

    #[test]
    fn cheering_fans_no_cheerleaders() {
        assert_eq!(cheering_fans_total(4, 3, 0), 7);
    }

    // ── brilliant_coaching_total ─────────────────────────────────────────────

    #[test]
    fn brilliant_coaching_normal_coach() {
        assert_eq!(brilliant_coaching_total(4, 3, 2, false), 9);
    }

    #[test]
    fn brilliant_coaching_banned_coach_minus_1() {
        assert_eq!(brilliant_coaching_total(4, 3, 2, true), 8);
    }

    #[test]
    fn brilliant_coaching_no_assistants() {
        assert_eq!(brilliant_coaching_total(4, 3, 0, false), 7);
    }

    // ── gains_extra_reroll ───────────────────────────────────────────────────

    #[test]
    fn higher_total_wins() {
        assert!(gains_extra_reroll(8, 5));
    }

    #[test]
    fn lower_total_does_not_gain() {
        assert!(!gains_extra_reroll(5, 8));
    }

    #[test]
    fn tie_both_gain_reroll() {
        assert!(gains_extra_reroll(7, 7));
    }

    // ── combined scenarios ───────────────────────────────────────────────────

    #[test]
    fn cheering_fans_home_wins_scenario() {
        let home = cheering_fans_total(4, 5, 2); // 11
        let away = cheering_fans_total(3, 4, 1); // 8
        assert!(gains_extra_reroll(home, away));
        assert!(!gains_extra_reroll(away, home));
    }

    #[test]
    fn brilliant_coaching_tie_both_win() {
        let home = brilliant_coaching_total(3, 5, 2, false); // 10
        let away = brilliant_coaching_total(4, 4, 2, false); // 10
        assert!(gains_extra_reroll(home, away));
        assert!(gains_extra_reroll(away, home));
    }
}
