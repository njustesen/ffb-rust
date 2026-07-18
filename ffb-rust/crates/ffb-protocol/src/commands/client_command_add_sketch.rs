use ffb_model::enums::NetCommandId;
use ffb_model::model::sketch::sketch::Sketch;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandAddSketch`.
/// Java: `private Sketch sketch;` — the full `Sketch` (id, rgb, label, path) round-trips
/// on the wire (`IJsonOption.SKETCH.addTo(jsonObject, sketch.toJsonValue())`), which matters
/// on the server side: `ServerCommandHandlerAddSketch` relays this exact sketch (including
/// its initial color and first coordinate — see `ClientSketchManager.create()`/
/// `PathSketchOverlay` in Java) to other sessions via `ServerCommandAddSketches`.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandAddSketch {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `sketch`.
    pub sketch: Option<Sketch>,
}

impl ClientCommandAddSketch {
    pub fn new() -> Self { Self::default() }

    /// Java: `ClientCommandAddSketch(Sketch sketch)`.
    pub fn with_sketch(sketch: Sketch) -> Self {
        Self { entropy: None, sketch: Some(sketch) }
    }

    /// Convenience constructor for callers that only have a sketch id on hand
    /// (e.g. tests) — builds a `Sketch` with just the id set.
    pub fn with_sketch_id(sketch_id: impl Into<String>) -> Self {
        let mut sketch = Sketch::new();
        sketch.id = sketch_id.into();
        Self::with_sketch(sketch)
    }

    /// Java: `getSketch()`.
    pub fn get_sketch(&self) -> Option<&Sketch> { self.sketch.as_ref() }

    pub fn get_sketch_id(&self) -> Option<&str> { self.sketch.as_ref().map(|s| s.get_id()) }

    /// Java: `ClientCommandAddSketch.toJsonValue()` (calls `super.toJsonValue()` first).
    /// Java: `IJsonOption.SKETCH.addTo(jsonObject, sketch.toJsonValue())`, wire key "sketch".
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        if let Some(sketch) = &self.sketch {
            map.insert("sketch".to_string(), sketch.to_json_value());
        }
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandAddSketch.initFrom(source, jsonValue)` —
    /// `sketch = new Sketch(0).initFrom(source, IJsonOption.SKETCH.getFrom(source, jsonObject))`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            sketch: json.get("sketch").map(Sketch::from_json),
        }
    }
}

impl NetCommand for ClientCommandAddSketch {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientAddSketch
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sketch_id_stored() {
        let cmd = ClientCommandAddSketch::with_sketch_id("sk-123");
        assert_eq!(cmd.get_sketch_id(), Some("sk-123"));
    }

    #[test]
    fn default_is_none() {
        let cmd = ClientCommandAddSketch::new();
        assert!(cmd.sketch.is_none());
    }

    #[test]
    fn with_sketch_carries_rgb_label_and_path() {
        use ffb_model::types::FieldCoordinate;
        let mut sketch = Sketch::with_rgb(255);
        sketch.id = "sk-full".to_string();
        sketch.set_label("note");
        sketch.add_coordinate(FieldCoordinate::new(1, 2));
        let cmd = ClientCommandAddSketch::with_sketch(sketch);
        assert_eq!(cmd.get_sketch_id(), Some("sk-full"));
        assert_eq!(cmd.get_sketch().unwrap().get_rgb(), 255);
        assert_eq!(cmd.get_sketch().unwrap().get_label(), Some("note"));
        assert_eq!(cmd.get_sketch().unwrap().get_path(), &[FieldCoordinate::new(1, 2)]);
    }

    #[test]
    fn round_trip_preserves_rgb_label_and_path() {
        use ffb_model::types::FieldCoordinate;
        let mut sketch = Sketch::with_rgb(16);
        sketch.id = "sk-rt".to_string();
        sketch.set_label("lbl");
        sketch.add_coordinate(FieldCoordinate::new(5, 6));
        let cmd = ClientCommandAddSketch::with_sketch(sketch);
        let json = cmd.to_json_value();
        let restored = ClientCommandAddSketch::from_json(&json);
        let restored_sketch = restored.get_sketch().unwrap();
        assert_eq!(restored_sketch.get_id(), "sk-rt");
        assert_eq!(restored_sketch.get_rgb(), 16);
        assert_eq!(restored_sketch.get_label(), Some("lbl"));
        assert_eq!(restored_sketch.get_path(), &[FieldCoordinate::new(5, 6)]);
    }

    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandAddSketch::new()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandAddSketch::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandAddSketch::default());
        assert!(s.contains("ClientCommandAddSketch"));
    }

    #[test]
    fn get_id_is_client_add_sketch() {
        assert_eq!(ClientCommandAddSketch::new().get_id(), NetCommandId::ClientAddSketch);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_sketch_id() {
        let cmd = ClientCommandAddSketch::with_sketch_id("sk-123");
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientAddSketch");
        assert_eq!(json["sketch"]["id"], "sk-123");
    }

    #[test]
    fn round_trip_with_sketch_id_and_entropy() {
        let mut cmd = ClientCommandAddSketch::with_sketch_id("sk-9");
        cmd.entropy = Some(4);
        let json = cmd.to_json_value();
        let restored = ClientCommandAddSketch::from_json(&json);
        assert_eq!(restored.entropy, Some(4));
        assert_eq!(restored.get_sketch_id(), Some("sk-9"));
    }

    #[test]
    fn round_trip_with_default_data() {
        let cmd = ClientCommandAddSketch::new();
        let json = cmd.to_json_value();
        let restored = ClientCommandAddSketch::from_json(&json);
        assert!(restored.sketch.is_none());
    }
}
