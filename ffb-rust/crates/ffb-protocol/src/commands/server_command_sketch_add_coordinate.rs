use ffb_model::enums::NetCommandId;
use ffb_model::types::FieldCoordinate;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandSketchAddCoordinate`.
/// Adds a point to a sketch path on the field.
#[derive(Debug, Clone)]
pub struct ServerCommandSketchAddCoordinate {
    /// Java: base-class `ServerCommand.fCommandNr`.
    pub command_nr: i32,
    /// Java: `coach` — coach who owns the sketch.
    pub coach: String,
    /// Java: `sketchId` — the sketch being extended.
    pub sketch_id: String,
    /// Java: `coordinate` — the new field coordinate to add.
    pub coordinate: FieldCoordinate,
}

impl ServerCommandSketchAddCoordinate {
    pub fn new(
        coach: impl Into<String>,
        sketch_id: impl Into<String>,
        coordinate: FieldCoordinate,
    ) -> Self {
        Self { command_nr: 0, coach: coach.into(), sketch_id: sketch_id.into(), coordinate }
    }
    pub fn get_coach(&self) -> &str { &self.coach }
    pub fn get_sketch_id(&self) -> &str { &self.sketch_id }
    pub fn get_coordinate(&self) -> FieldCoordinate { self.coordinate }

    /// Java: `ServerCommandSketchAddCoordinate.toJsonValue()` — no
    /// `commandNr` on the wire; `sketchId` is written under the `id` key.
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "netCommandId": self.get_id().name(),
            "id": self.sketch_id,
            "coordinate": self.coordinate.to_json_value(),
            "coach": self.coach,
        })
    }

    /// Java: `ServerCommandSketchAddCoordinate.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            command_nr: 0,
            sketch_id: json.get("id").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
            coordinate: json.get("coordinate").and_then(FieldCoordinate::from_json).unwrap_or(FieldCoordinate::new(0, 0)),
            coach: json.get("coach").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        }
    }
}

impl Default for ServerCommandSketchAddCoordinate {
    fn default() -> Self {
        Self {
            command_nr: 0,
            coach: String::new(),
            sketch_id: String::new(),
            coordinate: FieldCoordinate::new(0, 0),
        }
    }
}

impl NetCommand for ServerCommandSketchAddCoordinate {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerSketchAddCoordinate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let cmd = ServerCommandSketchAddCoordinate::new("Alice", "sk1", FieldCoordinate::new(5, 3));
        assert_eq!(cmd.get_coach(), "Alice");
        assert_eq!(cmd.get_sketch_id(), "sk1");
        assert_eq!(cmd.get_coordinate(), FieldCoordinate::new(5, 3));
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ServerCommandSketchAddCoordinate::default()).is_empty());
    }


    #[test]
    fn clone_roundtrip() {
        let cmd = ServerCommandSketchAddCoordinate::default();
        let _ = cmd.clone();
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ServerCommandSketchAddCoordinate::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ServerCommandSketchAddCoordinate::default());
        assert!(s.contains("ServerCommandSketchAddCoordinate"));
    }

    #[test]
    fn get_id_is_server_sketch_add_coordinate() {
        assert_eq!(
            ServerCommandSketchAddCoordinate::default().get_id(),
            NetCommandId::ServerSketchAddCoordinate
        );
    }

    #[test]
    fn to_json_value_has_net_command_id_and_id_key() {
        let cmd = ServerCommandSketchAddCoordinate::new("Alice", "sk1", FieldCoordinate::new(5, 3));
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "serverSketchAddCoordinate");
        assert_eq!(json["id"], "sk1");
        assert_eq!(json["coach"], "Alice");
        assert_eq!(json["coordinate"], serde_json::json!([5, 3]));
    }

    #[test]
    fn round_trip_with_data() {
        let cmd = ServerCommandSketchAddCoordinate::new("Bob", "sk2", FieldCoordinate::new(1, 2));
        let json = cmd.to_json_value();
        let restored = ServerCommandSketchAddCoordinate::from_json(&json);
        assert_eq!(restored.coach, "Bob");
        assert_eq!(restored.sketch_id, "sk2");
        assert_eq!(restored.coordinate, FieldCoordinate::new(1, 2));
    }

    #[test]
    fn round_trip_with_default() {
        let cmd = ServerCommandSketchAddCoordinate::default();
        let json = cmd.to_json_value();
        let restored = ServerCommandSketchAddCoordinate::from_json(&json);
        assert!(restored.coach.is_empty());
        assert!(restored.sketch_id.is_empty());
        assert_eq!(restored.coordinate, FieldCoordinate::new(0, 0));
    }
}
