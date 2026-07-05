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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new_stores_player_id_and_coordinate() {
        let p = Pushback::new("p1", FieldCoordinate::new(2, 4));
        assert_eq!(p.get_player_id(), Some("p1"));
        assert_eq!(p.get_coordinate(), Some(FieldCoordinate::new(2, 4)));
    }
    #[test]
    fn transform_preserves_player_id() {
        let p = Pushback::new("p2", FieldCoordinate::new(2, 4));
        let t = p.transform();
        assert_eq!(t.get_player_id(), Some("p2"));
    }
}
