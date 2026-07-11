use serde::{Deserialize, Serialize};
use ffb_model::enums::NetCommandId;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommand`.
/// Abstract base for all server-originated commands. Carries a monotonically
/// increasing `command_nr` used for replay sequencing.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerCommand {
    /// Java: `fCommandNr` — monotonically increasing sequence number for replay.
    pub command_nr: i32,
}

impl ServerCommand {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_command_nr(command_nr: i32) -> Self {
        Self { command_nr }
    }

    pub fn get_command_nr(&self) -> i32 {
        self.command_nr
    }

    pub fn set_command_nr(&mut self, command_nr: i32) {
        self.command_nr = command_nr;
    }

    /// Java: `isReplayable()` — most server commands are replayable by default.
    pub fn is_replayable(&self) -> bool {
        true
    }

    /// Java: `ServerCommand` subclasses don't call a shared `super
    /// .toJsonValue()` (unlike `ClientCommand`) — each writes `netCommandId`
    /// + `commandNr` by hand. This helper exists so the ~33 `ServerCommand*`
    /// leaf structs don't repeat those two keys, without changing the wire
    /// shape those classes actually produce.
    pub fn base_json_fields(&self, id: NetCommandId) -> serde_json::Map<String, serde_json::Value> {
        let mut map = serde_json::Map::new();
        map.insert("netCommandId".to_string(), serde_json::json!(id.name()));
        map.insert("commandNr".to_string(), serde_json::json!(self.command_nr));
        map
    }

    pub fn base_from_json(json: &serde_json::Value) -> Self {
        Self {
            command_nr: json.get("commandNr").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_command_nr_is_zero() {
        assert_eq!(ServerCommand::new().command_nr, 0);
    }

    #[test]
    fn with_command_nr_sets_field() {
        let cmd = ServerCommand::with_command_nr(7);
        assert_eq!(cmd.command_nr, 7);
    }

    #[test]
    fn is_replayable_default_true() {
        assert!(ServerCommand::new().is_replayable());
    }

    #[test]
    fn serde_round_trip() {
        let cmd = ServerCommand::with_command_nr(42);
        let json = serde_json::to_string(&cmd).unwrap();
        let back: ServerCommand = serde_json::from_str(&json).unwrap();
        assert_eq!(back.command_nr, 42);
    }

    #[test]
    fn base_json_fields_includes_net_command_id_and_command_nr() {
        let cmd = ServerCommand::with_command_nr(5);
        let fields = cmd.base_json_fields(NetCommandId::ServerGameTime);
        assert_eq!(fields["netCommandId"], "serverGameTime");
        assert_eq!(fields["commandNr"], 5);
    }

    #[test]
    fn base_from_json_round_trip() {
        let cmd = ServerCommand::with_command_nr(11);
        let json = serde_json::Value::Object(cmd.base_json_fields(NetCommandId::ServerGameTime));
        let restored = ServerCommand::base_from_json(&json);
        assert_eq!(restored.command_nr, 11);
    }
}
