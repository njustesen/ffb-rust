use ffb_model::enums::NetCommandId;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandRemoveSketches`.
/// Instructs clients to remove specific sketches by ID.
#[derive(Debug, Clone, Default)]
pub struct ServerCommandRemoveSketches {
    /// Java: base-class `ServerCommand.fCommandNr`.
    pub command_nr: i32,
    /// Java: `coach` — the coach whose sketches are being removed.
    pub coach: String,
    /// Java: `ids` — sketch IDs to remove.
    pub ids: Vec<String>,
}

impl ServerCommandRemoveSketches {
    pub fn new(coach: impl Into<String>, ids: Vec<String>) -> Self {
        Self { command_nr: 0, coach: coach.into(), ids }
    }
    pub fn get_coach(&self) -> &str { &self.coach }
    pub fn get_ids(&self) -> &[String] { &self.ids }

    /// Java: `ServerCommandRemoveSketches.toJsonValue()` — no `commandNr` on
    /// the wire; `ids` is only written when non-empty.
    pub fn to_json_value(&self) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        map.insert("netCommandId".to_string(), serde_json::json!(self.get_id().name()));
        if !self.ids.is_empty() {
            map.insert("ids".to_string(), serde_json::json!(self.ids));
        }
        map.insert("coach".to_string(), serde_json::json!(self.coach));
        serde_json::Value::Object(map)
    }

    /// Java: `ServerCommandRemoveSketches.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let ids = json
            .get("ids")
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();
        Self {
            command_nr: 0,
            coach: json.get("coach").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
            ids,
        }
    }
}

impl NetCommand for ServerCommandRemoveSketches {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerRemoveSketches
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let cmd = ServerCommandRemoveSketches::new("Bob", vec!["id1".into()]);
        assert_eq!(cmd.get_coach(), "Bob");
        assert_eq!(cmd.get_ids(), &["id1"]);
    }

    #[test]
    fn default_empty() {
        let cmd = ServerCommandRemoveSketches::default();
        assert!(cmd.ids.is_empty());
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ServerCommandRemoveSketches::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ServerCommandRemoveSketches::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ServerCommandRemoveSketches::default());
        assert!(s.contains("ServerCommandRemoveSketches"));
    }

    #[test]
    fn get_id_is_server_remove_sketches() {
        assert_eq!(ServerCommandRemoveSketches::default().get_id(), NetCommandId::ServerRemoveSketches);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_ids() {
        let cmd = ServerCommandRemoveSketches::new("Bob", vec!["id1".into()]);
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "serverRemoveSketches");
        assert_eq!(json["coach"], "Bob");
        assert_eq!(json["ids"][0], "id1");
    }

    #[test]
    fn to_json_value_omits_ids_when_empty() {
        let cmd = ServerCommandRemoveSketches::new("Bob", vec![]);
        let json = cmd.to_json_value();
        assert!(json.get("ids").is_none());
    }

    #[test]
    fn round_trip_with_data() {
        let cmd = ServerCommandRemoveSketches::new("Carol", vec!["a".into(), "b".into()]);
        let json = cmd.to_json_value();
        let restored = ServerCommandRemoveSketches::from_json(&json);
        assert_eq!(restored.coach, "Carol");
        assert_eq!(restored.ids, vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn round_trip_with_default() {
        let cmd = ServerCommandRemoveSketches::default();
        let json = cmd.to_json_value();
        let restored = ServerCommandRemoveSketches::from_json(&json);
        assert!(restored.coach.is_empty());
        assert!(restored.ids.is_empty());
    }
}
