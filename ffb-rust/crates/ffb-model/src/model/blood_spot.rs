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
