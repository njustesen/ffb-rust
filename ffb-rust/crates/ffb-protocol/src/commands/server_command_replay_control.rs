use ffb_model::enums::NetCommandId;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandReplayControl`.
/// Tells the client which coach is controlling replay playback.
#[derive(Debug, Clone, Default)]
pub struct ServerCommandReplayControl {
    /// Java: base-class `ServerCommand.fCommandNr`.
    pub command_nr: i32,
    /// Java: `coach` — coach name who controls the replay.
    pub coach: String,
}

impl ServerCommandReplayControl {
    pub fn new(coach: impl Into<String>) -> Self { Self { command_nr: 0, coach: coach.into() } }
    pub fn get_coach(&self) -> &str { &self.coach }

    /// Java: `ServerCommandReplayControl.toJsonValue()` — no `commandNr` on
    /// the wire.
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "netCommandId": self.get_id().name(),
            "coach": self.coach,
        })
    }

    /// Java: `ServerCommandReplayControl.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            command_nr: 0,
            coach: json.get("coach").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        }
    }
}

impl NetCommand for ServerCommandReplayControl {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerReplayControl
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coach_stored() {
        let cmd = ServerCommandReplayControl::new("Alice");
        assert_eq!(cmd.get_coach(), "Alice");
    }

    #[test]
    fn default_empty() {
        let cmd = ServerCommandReplayControl::default();
        assert!(cmd.coach.is_empty());
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ServerCommandReplayControl::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ServerCommandReplayControl::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ServerCommandReplayControl::default());
        assert!(s.contains("ServerCommandReplayControl"));
    }

    #[test]
    fn get_id_is_server_replay_control() {
        assert_eq!(ServerCommandReplayControl::new("A").get_id(), NetCommandId::ServerReplayControl);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_coach() {
        let cmd = ServerCommandReplayControl::new("Alice");
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "serverReplayControl");
        assert_eq!(json["coach"], "Alice");
    }

    #[test]
    fn round_trip_with_data() {
        let cmd = ServerCommandReplayControl::new("Bob");
        let json = cmd.to_json_value();
        let restored = ServerCommandReplayControl::from_json(&json);
        assert_eq!(restored.coach, "Bob");
    }

    #[test]
    fn round_trip_with_default() {
        let cmd = ServerCommandReplayControl::default();
        let json = cmd.to_json_value();
        let restored = ServerCommandReplayControl::from_json(&json);
        assert!(restored.coach.is_empty());
    }
}
