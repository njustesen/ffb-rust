use serde::{Deserialize, Serialize};
use crate::types::FieldCoordinate;

/// 1:1 translation of com.fumbbl.ffb.TrackNumber.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrackNumber {
    pub coordinate: Option<FieldCoordinate>,
    pub number: i32,
}

impl TrackNumber {
    pub fn new(coordinate: FieldCoordinate, number: i32) -> Self {
        TrackNumber { coordinate: Some(coordinate), number }
    }

    pub fn get_coordinate(&self) -> Option<&FieldCoordinate> { self.coordinate.as_ref() }
    pub fn get_number(&self) -> i32 { self.number }
}

impl Default for TrackNumber {
    fn default() -> Self {
        TrackNumber { coordinate: None, number: 0 }
    }
}
