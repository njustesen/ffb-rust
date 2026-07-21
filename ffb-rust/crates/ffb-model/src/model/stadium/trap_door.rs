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
