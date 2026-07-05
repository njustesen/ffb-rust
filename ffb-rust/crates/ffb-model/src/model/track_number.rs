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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new_stores_coordinate_and_number() {
        let t = TrackNumber::new(FieldCoordinate::new(5, 3), 7);
        assert_eq!(t.get_number(), 7);
        assert_eq!(t.get_coordinate(), Some(&FieldCoordinate::new(5, 3)));
    }
    #[test]
    fn default_is_zero_and_none() {
        let t = TrackNumber::default();
        assert_eq!(t.get_number(), 0);
        assert!(t.get_coordinate().is_none());
    }
}
