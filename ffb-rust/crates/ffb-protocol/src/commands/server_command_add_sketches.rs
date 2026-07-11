use ffb_model::model::sketch::sketch::Sketch;
use ffb_model::enums::NetCommandId;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandAddSketches`.
/// Sends one or more sketches from a coach to all clients.
#[derive(Debug, Clone, Default)]
pub struct ServerCommandAddSketches {
    /// Java: base-class `ServerCommand.fCommandNr`.
    pub command_nr: i32,
    /// Java: `coach` — the coach who drew the sketches.
    pub coach: String,
    /// Java: `sketches` — the list of sketch objects.
    pub sketches: Vec<Sketch>,
}

impl ServerCommandAddSketches {
    pub fn new(coach: impl Into<String>, sketches: Vec<Sketch>) -> Self {
        Self { command_nr: 0, coach: coach.into(), sketches }
    }
    pub fn get_coach(&self) -> &str { &self.coach }
    pub fn get_sketches(&self) -> &[Sketch] { &self.sketches }

    /// Java: `ServerCommandAddSketches.toJsonValue()`. Note the Java class
    /// does not call `IJsonOption.COMMAND_NR.addTo(...)` here (unlike most
    /// other `ServerCommand*` subclasses) — matched literally, `commandNr`
    /// is intentionally absent from the wire payload.
    pub fn to_json_value(&self) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        map.insert("netCommandId".to_string(), serde_json::json!(self.get_id().name()));
        let sketches: Vec<serde_json::Value> = self.sketches.iter().map(Sketch::to_json_value).collect();
        map.insert("sketches".to_string(), serde_json::Value::Array(sketches));
        map.insert("coach".to_string(), serde_json::json!(self.coach));
        serde_json::Value::Object(map)
    }

    /// Java: `ServerCommandAddSketches.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let sketches = json
            .get("sketches")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().map(Sketch::from_json).collect())
            .unwrap_or_default();
        Self {
            command_nr: 0,
            coach: json.get("coach").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
            sketches,
        }
    }
}

impl NetCommand for ServerCommandAddSketches {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerAddSketches
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let cmd = ServerCommandAddSketches::new("Alice", vec![Sketch::new()]);
        assert_eq!(cmd.get_coach(), "Alice");
        assert_eq!(cmd.get_sketches().len(), 1);
    }

    #[test]
    fn default_empty() {
        let cmd = ServerCommandAddSketches::default();
        assert!(cmd.coach.is_empty());
        assert!(cmd.sketches.is_empty());
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ServerCommandAddSketches::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ServerCommandAddSketches::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ServerCommandAddSketches::default());
        assert!(s.contains("ServerCommandAddSketches"));
    }

    #[test]
    fn get_id_is_server_add_sketches() {
        assert_eq!(ServerCommandAddSketches::default().get_id(), NetCommandId::ServerAddSketches);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_coach_and_no_command_nr() {
        let cmd = ServerCommandAddSketches::new("Alice", vec![]);
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "serverAddSketches");
        assert_eq!(json["coach"], "Alice");
        assert!(json.get("commandNr").is_none());
    }

    #[test]
    fn round_trip_with_sketches() {
        use ffb_model::types::FieldCoordinate;
        let mut sketch = Sketch::new();
        sketch.add_position(FieldCoordinate::new(2, 3));
        let cmd = ServerCommandAddSketches::new("Bob", vec![sketch]);
        let json = cmd.to_json_value();
        let restored = ServerCommandAddSketches::from_json(&json);
        assert_eq!(restored.coach, "Bob");
        assert_eq!(restored.sketches.len(), 1);
        assert_eq!(restored.sketches[0].len(), 1);
    }

    #[test]
    fn round_trip_with_empty_sketches() {
        let cmd = ServerCommandAddSketches::default();
        let json = cmd.to_json_value();
        let restored = ServerCommandAddSketches::from_json(&json);
        assert!(restored.coach.is_empty());
        assert!(restored.sketches.is_empty());
    }
}
