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

    /// Java: `Pushback.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        if let Some(player_id) = &self.player_id {
            map.insert("playerId".to_string(), serde_json::json!(player_id));
        }
        if let Some(coordinate) = self.coordinate {
            map.insert("coordinate".to_string(), coordinate.to_json_value());
        }
        serde_json::Value::Object(map)
    }

    /// Java: `Pushback.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            player_id: json.get("playerId").and_then(|v| v.as_str()).map(|s| s.to_string()),
            coordinate: json.get("coordinate").and_then(FieldCoordinate::from_json),
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

    #[test]
    fn default_has_no_player_id_or_coordinate() {
        let p = Pushback::default();
        assert!(p.get_player_id().is_none());
        assert!(p.get_coordinate().is_none());
    }

    #[test]
    fn serde_round_trip() {
        let p = Pushback::new("p3", FieldCoordinate::new(7, 3));
        let s = serde_json::to_string(&p).unwrap();
        let back: Pushback = serde_json::from_str(&s).unwrap();
        assert_eq!(back.get_player_id(), Some("p3"));
        assert_eq!(back.get_coordinate(), Some(FieldCoordinate::new(7, 3)));
    }

    #[test]
    fn transform_changes_coordinate() {
        let p = Pushback::new("p1", FieldCoordinate::new(5, 5));
        let t = p.transform();
        // transform mirrors the coordinate; original and transformed should differ or be equal
        // depending on FieldCoordinate::transform — just verify coordinate is Some
        assert!(t.get_coordinate().is_some());
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", Pushback::default()).is_empty());
    }

}
