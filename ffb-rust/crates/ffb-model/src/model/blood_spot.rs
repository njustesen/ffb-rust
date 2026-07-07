use serde::{Deserialize, Serialize};
use crate::enums::PlayerState;
use crate::types::FieldCoordinate;

/// 1:1 translation of com.fumbbl.ffb.BloodSpot.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BloodSpot {
    pub injury: Option<PlayerState>,
    pub coordinate: Option<FieldCoordinate>,
}

impl BloodSpot {
    pub fn new(coordinate: FieldCoordinate, injury: PlayerState) -> Self {
        BloodSpot { coordinate: Some(coordinate), injury: Some(injury) }
    }

    pub fn get_injury(&self) -> Option<PlayerState> { self.injury }
    pub fn get_coordinate(&self) -> Option<FieldCoordinate> { self.coordinate }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_has_no_fields() {
        let b = BloodSpot::default();
        assert!(b.injury.is_none());
        assert!(b.coordinate.is_none());
    }
    #[test]
    fn new_stores_coordinate_and_injury() {
        use crate::enums::PS_PRONE;
        let state = PlayerState::new(PS_PRONE);
        let b = BloodSpot::new(FieldCoordinate::new(3, 7), state);
        assert_eq!(b.get_coordinate(), Some(FieldCoordinate::new(3, 7)));
        assert!(b.get_injury().is_some_and(|s| s.is_prone()));
    }

    #[test]
    fn serde_round_trip_default() {
        let b = BloodSpot::default();
        let s = serde_json::to_string(&b).unwrap();
        let back: BloodSpot = serde_json::from_str(&s).unwrap();
        assert!(back.injury.is_none());
        assert!(back.coordinate.is_none());
    }

    #[test]
    fn new_different_coordinates() {
        use crate::enums::{PS_PRONE, PlayerState};
        let state = PlayerState::new(PS_PRONE);
        let b1 = BloodSpot::new(FieldCoordinate::new(0, 0), state);
        let b2 = BloodSpot::new(FieldCoordinate::new(10, 5), state);
        assert_ne!(b1.get_coordinate(), b2.get_coordinate());
    }

    #[test]
    fn get_injury_returns_stored_state() {
        use crate::enums::{PS_STANDING, PlayerState};
        let state = PlayerState::new(PS_STANDING);
        let b = BloodSpot::new(FieldCoordinate::new(1, 1), state);
        assert!(b.get_injury().is_some());
    }
}
