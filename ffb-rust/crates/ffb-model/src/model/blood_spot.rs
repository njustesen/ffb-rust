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
    fn serde_round_trip_default() {
        let b = BloodSpot::default();
        let s = serde_json::to_string(&b).unwrap();
        let back: BloodSpot = serde_json::from_str(&s).unwrap();
        assert!(back.injury.is_none());
        assert!(back.coordinate.is_none());
    }

}
