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

// Java-derived cases (KickoffEventCalcTest) live in the 1:1 mirror module
// ffb-engine/src/util/kickoff_event_calc.rs; the former tests here duplicated
// them exactly (same inputs and expectations) and were removed.
