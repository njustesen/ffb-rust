use serde::{Deserialize, Serialize};
use crate::types::FieldCoordinate;

/// 1:1 translation of com.fumbbl.ffb.model.Sketch.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Sketch {
    pub positions: Vec<FieldCoordinate>,
}

impl Sketch {
    pub fn new() -> Self { Self::default() }
    pub fn add_position(&mut self, pos: FieldCoordinate) { self.positions.push(pos); }
    pub fn len(&self) -> usize { self.positions.len() }
    pub fn is_empty(&self) -> bool { self.positions.is_empty() }

    /// Java: `Sketch.toJsonValue()`. The Rust struct only models `path`
    /// (as `positions`) — `id`/`rgb`/`label` were never ported onto this
    /// struct, so only `fieldCoordinates` is emitted here.
    pub fn to_json_value(&self) -> serde_json::Value {
        let coords: Vec<serde_json::Value> = self.positions.iter().map(|c| c.to_json_value()).collect();
        serde_json::json!({ "fieldCoordinates": coords })
    }

    /// Java: `Sketch.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let positions = json
            .get("fieldCoordinates")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(FieldCoordinate::from_json).collect())
            .unwrap_or_default();
        Self { positions }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_by_default() {
        assert!(Sketch::new().is_empty());
    }

    #[test]
    fn add_position_increases_len() {
        let mut s = Sketch::new();
        s.add_position(FieldCoordinate::new(0, 0));
        assert_eq!(s.len(), 1);
    }
}
