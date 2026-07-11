use ffb_model::enums::NetCommandId;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandReplayStatus`.
/// Communicates replay playback state to the client.
/// Java note: `commandNr` here is a *local* field distinct from the
/// `ServerCommand.fCommandNr` base field (the base one is never set/read by
/// this class); the Rust struct's single `command_nr` field maps to that
/// local field.
#[derive(Debug, Clone, Default)]
pub struct ServerCommandReplayStatus {
    /// Java: `commandNr` — current replay command index.
    pub command_nr: i32,
    /// Java: `speed` — playback speed multiplier.
    pub speed: i32,
    /// Java: `running` — whether replay is actively playing.
    pub running: bool,
    /// Java: `forward` — playback direction (true = forward).
    pub forward: bool,
    /// Java: `skip` — whether skipping to a position.
    pub skip: bool,
}

impl ServerCommandReplayStatus {
    pub fn new(command_nr: i32, speed: i32, running: bool, forward: bool, skip: bool) -> Self {
        Self { command_nr, speed, running, forward, skip }
    }
    pub fn get_command_nr(&self) -> i32 { self.command_nr }
    pub fn get_speed(&self) -> i32 { self.speed }
    pub fn is_running(&self) -> bool { self.running }
    pub fn is_forward(&self) -> bool { self.forward }
    pub fn is_skip(&self) -> bool { self.skip }

    /// Java: `ServerCommandReplayStatus.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "netCommandId": self.get_id().name(),
            "commandNr": self.command_nr,
            "running": self.running,
            "forward": self.forward,
            "speed": self.speed,
            "skip": self.skip,
        })
    }

    /// Java: `ServerCommandReplayStatus.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            forward: json.get("forward").and_then(|v| v.as_bool()).unwrap_or(false),
            running: json.get("running").and_then(|v| v.as_bool()).unwrap_or(false),
            command_nr: json.get("commandNr").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            speed: json.get("speed").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            skip: json.get("skip").and_then(|v| v.as_bool()).unwrap_or(false),
        }
    }
}

impl NetCommand for ServerCommandReplayStatus {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerReplayStatus
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_fields_stored() {
        let cmd = ServerCommandReplayStatus::new(42, 2, true, true, false);
        assert_eq!(cmd.get_command_nr(), 42);
        assert_eq!(cmd.get_speed(), 2);
        assert!(cmd.is_running());
        assert!(cmd.is_forward());
        assert!(!cmd.is_skip());
    }

    #[test]
    fn default_is_stopped() {
        let cmd = ServerCommandReplayStatus::default();
        assert!(!cmd.running);
        assert_eq!(cmd.command_nr, 0);
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ServerCommandReplayStatus::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ServerCommandReplayStatus::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ServerCommandReplayStatus::default());
        assert!(s.contains("ServerCommandReplayStatus"));
    }

    #[test]
    fn get_id_is_server_replay_status() {
        assert_eq!(ServerCommandReplayStatus::default().get_id(), NetCommandId::ServerReplayStatus);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_fields() {
        let cmd = ServerCommandReplayStatus::new(42, 2, true, true, false);
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "serverReplayStatus");
        assert_eq!(json["commandNr"], 42);
        assert_eq!(json["speed"], 2);
        assert_eq!(json["running"], true);
        assert_eq!(json["forward"], true);
        assert_eq!(json["skip"], false);
    }

    #[test]
    fn round_trip_with_data() {
        let cmd = ServerCommandReplayStatus::new(9, 3, true, false, true);
        let json = cmd.to_json_value();
        let restored = ServerCommandReplayStatus::from_json(&json);
        assert_eq!(restored.command_nr, 9);
        assert_eq!(restored.speed, 3);
        assert!(restored.running);
        assert!(!restored.forward);
        assert!(restored.skip);
    }

    #[test]
    fn round_trip_with_default() {
        let cmd = ServerCommandReplayStatus::default();
        let json = cmd.to_json_value();
        let restored = ServerCommandReplayStatus::from_json(&json);
        assert_eq!(restored.command_nr, 0);
        assert!(!restored.running);
    }
}
