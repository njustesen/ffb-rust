use std::collections::HashMap;
use ffb_model::enums::NetCommandId;
use crate::commands::server_command::ServerCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandAutomaticPlayerMarkings`.
/// Sends automatic player markings (player_id → marking colour) to the client.
#[derive(Debug, Clone, Default)]
pub struct ServerCommandAutomaticPlayerMarkings {
    /// Java: base-class `ServerCommand.fCommandNr`.
    pub command_nr: i32,
    /// Java: `markings` — map of player_id → marking colour string.
    pub markings: HashMap<String, String>,
    /// Java: `index` — which markings set index this applies to.
    pub index: i32,
}

impl ServerCommandAutomaticPlayerMarkings {
    pub fn new(markings: HashMap<String, String>, index: i32) -> Self {
        Self { command_nr: 0, markings, index }
    }
    pub fn get_markings(&self) -> &HashMap<String, String> { &self.markings }
    pub fn get_index(&self) -> i32 { self.index }

    /// Java: `isReplayable()`.
    pub fn is_replayable(&self) -> bool {
        false
    }

    /// Java: `ServerCommandAutomaticPlayerMarkings.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ServerCommand { command_nr: self.command_nr };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("markings".to_string(), serde_json::json!(self.markings));
        map.insert("selectedIndex".to_string(), serde_json::json!(self.index));
        serde_json::Value::Object(map)
    }

    /// Java: `ServerCommandAutomaticPlayerMarkings.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ServerCommand::base_from_json(json);
        let markings = json
            .get("markings")
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or_default();
        Self {
            command_nr: base.command_nr,
            markings,
            index: json.get("selectedIndex").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
        }
    }
}

impl NetCommand for ServerCommandAutomaticPlayerMarkings {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerAutomaticPlayerMarkings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let mut map = HashMap::new();
        map.insert("p1".into(), "red".into());
        let cmd = ServerCommandAutomaticPlayerMarkings::new(map.clone(), 2);
        assert_eq!(cmd.get_index(), 2);
        assert_eq!(cmd.get_markings().get("p1").map(|s| s.as_str()), Some("red"));
    }

    #[test]
    fn default_empty() {
        let cmd = ServerCommandAutomaticPlayerMarkings::default();
        assert!(cmd.markings.is_empty());
        assert_eq!(cmd.index, 0);
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ServerCommandAutomaticPlayerMarkings::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ServerCommandAutomaticPlayerMarkings::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ServerCommandAutomaticPlayerMarkings::default());
        assert!(s.contains("ServerCommandAutomaticPlayerMarkings"));
    }

    #[test]
    fn get_id_is_server_automatic_player_markings() {
        assert_eq!(
            ServerCommandAutomaticPlayerMarkings::default().get_id(),
            NetCommandId::ServerAutomaticPlayerMarkings
        );
    }

    #[test]
    fn to_json_value_has_net_command_id_and_selected_index() {
        let mut map = HashMap::new();
        map.insert("p1".into(), "red".into());
        let cmd = ServerCommandAutomaticPlayerMarkings::new(map, 3);
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "serverAutomaticPlayerMarkings");
        assert_eq!(json["selectedIndex"], 3);
        assert_eq!(json["markings"]["p1"], "red");
    }

    #[test]
    fn round_trip_with_markings() {
        let mut map = HashMap::new();
        map.insert("p1".into(), "red".into());
        map.insert("p2".into(), "blue".into());
        let mut cmd = ServerCommandAutomaticPlayerMarkings::new(map, 1);
        cmd.command_nr = 7;
        let json = cmd.to_json_value();
        let restored = ServerCommandAutomaticPlayerMarkings::from_json(&json);
        assert_eq!(restored.command_nr, 7);
        assert_eq!(restored.index, 1);
        assert_eq!(restored.markings.get("p1").map(|s| s.as_str()), Some("red"));
        assert_eq!(restored.markings.get("p2").map(|s| s.as_str()), Some("blue"));
    }

    #[test]
    fn round_trip_with_no_markings() {
        let cmd = ServerCommandAutomaticPlayerMarkings::default();
        let json = cmd.to_json_value();
        let restored = ServerCommandAutomaticPlayerMarkings::from_json(&json);
        assert!(restored.markings.is_empty());
        assert_eq!(restored.index, 0);
    }
}
