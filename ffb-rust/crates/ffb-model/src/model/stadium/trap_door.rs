use serde::{Deserialize, Serialize};
use crate::types::FieldCoordinate;

/// 1:1 translation of com.fumbbl.ffb.model.stadium.TrapDoor.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TrapDoor {
    pub coordinate: Option<FieldCoordinate>,
    pub active: bool,
}

impl TrapDoor {
    pub fn new(coordinate: FieldCoordinate) -> Self {
        Self { coordinate: Some(coordinate), active: true }
    }
    pub fn get_coordinate(&self) -> Option<&FieldCoordinate> { self.coordinate.as_ref() }
    pub fn is_active(&self) -> bool { self.active }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::FieldCoordinate;

    #[test]
    fn default_is_not_active() {
        assert!(!TrapDoor::default().is_active());
    }

    #[test]
    fn new_is_active() {
        let t = TrapDoor::new(FieldCoordinate::new(5, 5));
        assert!(t.is_active());
        assert!(t.get_coordinate().is_some());
    }
}
