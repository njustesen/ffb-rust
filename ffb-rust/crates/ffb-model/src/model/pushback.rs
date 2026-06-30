use serde::{Deserialize, Serialize};
use crate::types::FieldCoordinate;

/// 1:1 translation of com.fumbbl.ffb.Pushback.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Pushback {
    pub player_id: Option<String>,
    pub coordinate: Option<FieldCoordinate>,
}

impl Pushback {
    pub fn new(player_id: impl Into<String>, coordinate: FieldCoordinate) -> Self {
        Pushback { player_id: Some(player_id.into()), coordinate: Some(coordinate) }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_coordinate(&self) -> Option<FieldCoordinate> { self.coordinate }

    pub fn transform(&self) -> Pushback {
        Pushback {
            player_id: self.player_id.clone(),
            coordinate: self.coordinate.map(|c| c.transform()),
        }
    }
}
