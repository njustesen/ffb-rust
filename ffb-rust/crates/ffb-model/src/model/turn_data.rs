use serde::{Deserialize, Serialize};
use crate::enums::LeaderState;

/// Per-team per-half turn state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnData {
    pub turn_nr: i32,
    pub first_turn_after_kickoff: bool,
    pub turn_started: bool,

    pub rerolls: i32,
    pub single_use_rerolls: i32,
    pub rerolls_brilliant_coaching_one_drive: i32,
    pub reroll_show_star_one_drive: i32,
    pub rerolls_pump_up_the_crowd_one_drive: i32,

    pub apothecaries: i32,
    pub wandering_apothecaries: i32,
    pub plague_doctors: i32,

    pub blitz_used: bool,
    pub foul_used: bool,
    pub reroll_used: bool,
    pub hand_over_used: bool,
    pub pass_used: bool,
    pub ttm_used: bool,
    pub ktm_used: bool,
    pub bomb_used: bool,
    pub secure_the_ball_used: bool,
    pub punt_used: bool,
    pub coach_banned: bool,

    pub leader_state: LeaderState,
    pub lord_of_chaos_state: LeaderState,
    /// QuickSnap kickoff event bonus: +1 MA for all players this turn.
    pub quick_snap_bonus: i32,
}

impl TurnData {
    pub fn new() -> Self {
        TurnData {
            turn_nr: 0,
            first_turn_after_kickoff: false,
            turn_started: false,
            rerolls: 0,
            single_use_rerolls: 0,
            rerolls_brilliant_coaching_one_drive: 0,
            reroll_show_star_one_drive: 0,
            rerolls_pump_up_the_crowd_one_drive: 0,
            apothecaries: 0,
            wandering_apothecaries: 0,
            plague_doctors: 0,
            blitz_used: false,
            foul_used: false,
            reroll_used: false,
            hand_over_used: false,
            pass_used: false,
            ttm_used: false,
            ktm_used: false,
            bomb_used: false,
            secure_the_ball_used: false,
            punt_used: false,
            coach_banned: false,
            leader_state: LeaderState::None,
            lord_of_chaos_state: LeaderState::None,
            quick_snap_bonus: 0,
        }
    }

    pub fn reset_for_turn(&mut self) {
        self.blitz_used = false;
        self.foul_used = false;
        self.reroll_used = false;
        self.hand_over_used = false;
        self.pass_used = false;
        self.ttm_used = false;
        self.ktm_used = false;
        self.bomb_used = false;
        self.secure_the_ball_used = false;
        self.punt_used = false;
        self.quick_snap_bonus = 0;
    }
}

impl Default for TurnData {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_defaults() {
        let td = TurnData::new();
        assert_eq!(td.turn_nr, 0);
        assert!(!td.blitz_used);
        assert_eq!(td.leader_state, LeaderState::None);
    }

    #[test]
    fn reset_clears_flags() {
        let mut td = TurnData::new();
        td.blitz_used = true;
        td.pass_used = true;
        td.reset_for_turn();
        assert!(!td.blitz_used);
        assert!(!td.pass_used);
    }

    #[test]
    fn serde_round_trip() {
        let td = TurnData::new();
        let json = serde_json::to_string(&td).unwrap();
        let back: TurnData = serde_json::from_str(&json).unwrap();
        assert_eq!(td.turn_nr, back.turn_nr);
        assert_eq!(td.leader_state, back.leader_state);
    }

    #[test]
    fn reset_does_not_clear_rerolls() {
        let mut td = TurnData::new();
        td.rerolls = 3;
        td.apothecaries = 1;
        td.blitz_used = true;
        td.reset_for_turn();
        assert_eq!(td.rerolls, 3);
        assert_eq!(td.apothecaries, 1);
        assert!(!td.blitz_used);
    }

    #[test]
    fn quick_snap_bonus_is_reset_on_turn_end() {
        let mut td = TurnData::new();
        td.quick_snap_bonus = 1;
        td.reset_for_turn();
        assert_eq!(td.quick_snap_bonus, 0);
    }

    #[test]
    fn all_action_flags_reset_together() {
        let mut td = TurnData::new();
        td.blitz_used = true;
        td.foul_used = true;
        td.pass_used = true;
        td.hand_over_used = true;
        td.ttm_used = true;
        td.bomb_used = true;
        td.secure_the_ball_used = true;
        td.punt_used = true;
        td.reset_for_turn();
        assert!(!td.foul_used);
        assert!(!td.pass_used);
        assert!(!td.hand_over_used);
        assert!(!td.ttm_used);
        assert!(!td.bomb_used);
        assert!(!td.secure_the_ball_used);
        assert!(!td.punt_used);
    }
}
