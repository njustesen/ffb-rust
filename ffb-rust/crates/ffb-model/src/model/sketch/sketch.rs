use serde::{Deserialize, Serialize};
use crate::types::FieldCoordinate;

/// 1:1 translation of `com.fumbbl.ffb.model.sketch.Sketch`.
/// Java: `id` (defaults to a random UUID), `rgb`, `label`, `path` (`positions` here).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Sketch {
    /// Java: `id` — normally assigned by the caller (a fresh id on creation, or the
    /// id parsed off the wire); left empty by `Default`/`new()` like the other
    /// blank-by-default model structs in this crate.
    pub id: String,
    /// Java: `rgb`.
    pub rgb: i32,
    /// Java: `label`.
    pub label: Option<String>,
    /// Java: `path`.
    pub positions: Vec<FieldCoordinate>,
}

impl Sketch {
    pub fn new() -> Self { Self::default() }

    /// Java: `Sketch(int rgb)` constructor.
    pub fn with_rgb(rgb: i32) -> Self {
        Self { rgb, ..Self::default() }
    }

    /// Java: `getPath()`.
    pub fn get_path(&self) -> &[FieldCoordinate] { &self.positions }

    /// Java: `getId()`.
    pub fn get_id(&self) -> &str { &self.id }

    /// Java: `getRgb()`.
    pub fn get_rgb(&self) -> i32 { self.rgb }

    /// Java: `setRgb(int)`.
    pub fn set_rgb(&mut self, rgb: i32) { self.rgb = rgb; }

    /// Java: `getLabel()`.
    pub fn get_label(&self) -> Option<&str> { self.label.as_deref() }

    /// Java: `setLabel(String)`.
    pub fn set_label(&mut self, label: impl Into<String>) { self.label = Some(label.into()); }

    /// Java: `addCoordinate(FieldCoordinate)` — only appends when it differs from the
    /// last coordinate already on the path.
    pub fn add_coordinate(&mut self, coordinate: FieldCoordinate) {
        if self.positions.last() != Some(&coordinate) {
            self.positions.push(coordinate);
        }
    }

    /// Kept for existing callers that built up a path with unconditional pushes
    /// before this struct had Java's dedup-on-add semantics.
    pub fn add_position(&mut self, pos: FieldCoordinate) { self.positions.push(pos); }
    pub fn len(&self) -> usize { self.positions.len() }
    pub fn is_empty(&self) -> bool { self.positions.is_empty() }

    /// Java: `Sketch.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let coords: Vec<serde_json::Value> = self.positions.iter().map(|c| c.to_json_value()).collect();
        serde_json::json!({
            "id": self.id,
            "rgb": self.rgb,
            "text": self.label,
            "fieldCoordinates": coords,
        })
    }

    /// Java: `Sketch.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let id = json.get("id").and_then(|v| v.as_str()).unwrap_or_default().to_string();
        let rgb = json.get("rgb").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let label = json.get("text").and_then(|v| v.as_str()).map(|s| s.to_string());
        let positions = json
            .get("fieldCoordinates")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(FieldCoordinate::from_json).collect())
            .unwrap_or_default();
        Self { id, rgb, label, positions }
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

    #[test]
    fn with_rgb_sets_rgb() {
        let s = Sketch::with_rgb(42);
        assert_eq!(s.get_rgb(), 42);
    }

    #[test]
    fn add_coordinate_dedups_consecutive_duplicates() {
        let mut s = Sketch::new();
        s.add_coordinate(FieldCoordinate::new(1, 1));
        s.add_coordinate(FieldCoordinate::new(1, 1));
        assert_eq!(s.len(), 1);
        s.add_coordinate(FieldCoordinate::new(2, 2));
        assert_eq!(s.len(), 2);
    }

    #[test]
    fn round_trip_with_id_rgb_label_and_path() {
        let mut s = Sketch::with_rgb(255);
        s.id = "sk-1".to_string();
        s.set_label("hello");
        s.add_coordinate(FieldCoordinate::new(3, 4));
        let json = s.to_json_value();
        let restored = Sketch::from_json(&json);
        assert_eq!(restored.get_id(), "sk-1");
        assert_eq!(restored.get_rgb(), 255);
        assert_eq!(restored.get_label(), Some("hello"));
        assert_eq!(restored.get_path(), &[FieldCoordinate::new(3, 4)]);
    }
}
