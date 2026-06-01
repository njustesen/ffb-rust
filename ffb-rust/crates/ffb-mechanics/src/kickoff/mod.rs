// Re-export kickoff event tables from ffb-model.
pub use ffb_model::kickoff::{
    kickoff_event, kickoff_event_bb2016, kickoff_event_bb2020, kickoff_event_bb2025,
    KickoffEventKind,
};


// ── Kickoff event roll calculations ──────────────────────────────────────────

/// Cheering Fans total for one team: D6 + fame + cheerleaders.
pub fn cheering_fans_total(die_roll: i32, fame: i32, cheerleaders: i32) -> i32 {
    die_roll + fame + cheerleaders
}

/// Brilliant Coaching total for one team: D6 + fame + assistant_coaches - (1 if coach banned).
pub fn brilliant_coaching_total(die_roll: i32, fame: i32, assistant_coaches: i32, coach_banned: bool) -> i32 {
    die_roll + fame + assistant_coaches + if coach_banned { -1 } else { 0 }
}

/// Whether a team gains an extra reroll from the event (own_total >= opponent_total).
/// Both teams gain a reroll when totals are tied.
pub fn gains_extra_reroll(own_total: i32, opponent_total: i32) -> bool {
    own_total >= opponent_total
}

/// Compute the number of players stunned by Pitch Invasion.
///
/// Each player on the pitch rolls a d6; stunned if ≤ fan_factor_advantage.
/// Returns the number of rolls that result in a stun.
///
/// This function is deterministic given the individual rolls — the caller
/// supplies the d6 results for each player.
pub fn pitch_invasion_stunned(rolls: &[i32], fan_factor_advantage: i32) -> usize {
    rolls.iter().filter(|&&r| r <= fan_factor_advantage).count()
}

/// Number of players the kicking/receiving team can reposition on Solid Defence / Quick Snap.
///
/// Formula: d3 + 3 (range 4–6 inclusive).
pub fn solid_defence_player_count(d3_roll: i32) -> i32 {
    d3_roll + 3
}

/// Whether the Time Out event moves the turn marker forward or back.
///
/// Returns `true` (forward/toward 8) when the kicking team is NOT on turns 6, 7, or 8.
/// The turn marker moves back by one if kicking team is on turn 6, 7, or 8.
pub fn time_out_moves_forward(kicking_team_turn: i32) -> bool {
    kicking_team_turn < 6
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cheering_fans_total_sums_correctly() {
        // D6=3, fame=2, cheerleaders=1 → 6
        assert_eq!(cheering_fans_total(3, 2, 1), 6);
        assert_eq!(cheering_fans_total(1, 0, 0), 1);
    }

    #[test]
    fn brilliant_coaching_total_subtracts_banned_coach() {
        // D6=4, fame=1, assistants=2 → 7; with banned: 6
        assert_eq!(brilliant_coaching_total(4, 1, 2, false), 7);
        assert_eq!(brilliant_coaching_total(4, 1, 2, true), 6);
    }

    #[test]
    fn gains_reroll_when_tied_or_higher() {
        assert!(gains_extra_reroll(5, 5));
        assert!(gains_extra_reroll(6, 5));
        assert!(!gains_extra_reroll(4, 5));
    }

    #[test]
    fn solid_defence_count() {
        assert_eq!(solid_defence_player_count(1), 4);
        assert_eq!(solid_defence_player_count(3), 6);
    }

    #[test]
    fn time_out_direction() {
        assert!(time_out_moves_forward(5));
        assert!(!time_out_moves_forward(6));
        assert!(!time_out_moves_forward(8));
    }

    #[test]
    fn pitch_invasion_stun_count() {
        // fan factor advantage of 2: rolls ≤ 2 are stunned
        assert_eq!(pitch_invasion_stunned(&[1, 2, 3, 4, 5, 6], 2), 2);
        assert_eq!(pitch_invasion_stunned(&[1, 1, 1], 0), 0);
    }
}
