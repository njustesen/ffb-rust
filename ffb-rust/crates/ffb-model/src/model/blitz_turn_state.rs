use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.model.BlitzTurnState.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlitzTurnState {
    pub amount: i32,
    pub available: i32,
    pub limit: i32,
    pub acting_player_was_changed: bool,
}

impl BlitzTurnState {
    pub fn new(limit: i32, available: i32) -> Self {
        BlitzTurnState { limit, available, ..Default::default() }
    }

    pub fn get_amount(&self) -> i32 { self.amount }
    pub fn get_limit(&self) -> i32 { self.limit }
    pub fn get_available(&self) -> i32 { self.available }

    pub fn add_activation(&mut self) {
        self.amount += 1;
        self.available -= 1;
    }

    pub fn limit_reached(&self) -> bool { self.amount == self.limit }
    pub fn available_players_left(&self) -> bool { self.available > 0 }
    pub fn acting_player_was_changed(&self) -> bool { self.acting_player_was_changed }
    pub fn change_acting_player(&mut self) { self.acting_player_was_changed = true; }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_activation_increments_amount_decrements_available() {
        let mut s = BlitzTurnState::new(3, 2);
        s.add_activation();
        assert_eq!(s.get_amount(), 1);
        assert_eq!(s.get_available(), 1);
    }

    #[test]
    fn limit_reached_when_amount_equals_limit() {
        let mut s = BlitzTurnState::new(2, 2);
        s.add_activation();
        assert!(!s.limit_reached());
        s.add_activation();
        assert!(s.limit_reached());
    }

    #[test]
    fn available_players_left_false_when_zero() {
        let mut s = BlitzTurnState::new(1, 1);
        s.add_activation();
        assert!(!s.available_players_left());
    }

    #[test]
    fn serde_round_trip() {
        let mut s = BlitzTurnState::new(3, 3);
        s.change_acting_player();
        let json = serde_json::to_string(&s).unwrap();
        let back: BlitzTurnState = serde_json::from_str(&json).unwrap();
        assert!(back.acting_player_was_changed());
        assert_eq!(back.get_limit(), 3);
    }

    #[test]
    fn new_sets_limit_and_available() {
        let s = BlitzTurnState::new(5, 3);
        assert_eq!(s.get_limit(), 5);
        assert_eq!(s.get_available(), 3);
        assert_eq!(s.get_amount(), 0);
    }

    #[test]
    fn change_acting_player_is_idempotent() {
        let mut s = BlitzTurnState::new(2, 2);
        s.change_acting_player();
        s.change_acting_player();
        assert!(s.acting_player_was_changed());
    }
}
