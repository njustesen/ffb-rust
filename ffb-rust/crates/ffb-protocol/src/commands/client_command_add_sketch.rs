use ffb_model::enums::NetCommandId;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandAddSketch`.
/// Java holds a `Sketch` which is a client-side rendering class (path data, color, label).
/// Full Sketch serialization is deferred; only the sketch ID is carried here for now.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandAddSketch {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Identifies the sketch being added. Full Sketch serialization is DEFERRED.
    pub sketch_id: Option<String>,
}

impl ClientCommandAddSketch {
    pub fn new() -> Self { Self::default() }

    pub fn with_sketch_id(sketch_id: impl Into<String>) -> Self {
        Self { entropy: None, sketch_id: Some(sketch_id.into()) }
    }

    pub fn get_sketch_id(&self) -> Option<&str> { self.sketch_id.as_deref() }

    /// Java: `ClientCommandAddSketch.toJsonValue()` (calls `super.toJsonValue()` first).
    /// NOTE: Java's `IJsonOption.SKETCH.addTo(jsonObject, sketch.toJsonValue())` serializes
    /// the full `Sketch` (id, rgb, label, path). The Rust `ClientCommandAddSketch` struct
    /// only carries `sketch_id` (full `Sketch` translation is out of scope here — see the
    /// struct doc comment), so only `sketch.id` (`IJsonOption.ID`, wire key "id") round-trips.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        if let Some(sketch_id) = &self.sketch_id {
            map.insert(
                "sketch".to_string(),
                serde_json::json!({ "id": sketch_id }),
            );
        }
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandAddSketch.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            sketch_id: json
                .get("sketch")
                .and_then(|v| v.get("id"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
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
        assert!(cmd.sketch_id.is_none());
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
        assert!(restored.sketch_id.is_none());
    }
}
