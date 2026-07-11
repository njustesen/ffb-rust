use ffb_model::enums::{NetCommandId, ServerStatus};
use crate::commands::server_command::ServerCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandStatus`.
/// Reports server connection status (error or success) to the client.
#[derive(Debug, Clone, Default)]
pub struct ServerCommandStatus {
    /// Java: base-class `ServerCommand.fCommandNr`.
    pub command_nr: i32,
    /// Java: `fServerStatus` — status code.
    pub server_status: Option<ServerStatus>,
    /// Java: `fMessage` — human-readable status message.
    pub message: String,
}

impl ServerCommandStatus {
    pub fn new(server_status: ServerStatus, message: impl Into<String>) -> Self {
        Self { command_nr: 0, server_status: Some(server_status), message: message.into() }
    }
    pub fn get_server_status(&self) -> Option<ServerStatus> { self.server_status }
    pub fn get_message(&self) -> &str { &self.message }

    /// Java: `ServerCommandStatus.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ServerCommand { command_nr: self.command_nr };
        let mut map = base.base_json_fields(self.get_id());
        if let Some(server_status) = self.server_status {
            map.insert("serverStatus".to_string(), serde_json::json!(server_status.name()));
        }
        map.insert("message".to_string(), serde_json::json!(self.message));
        serde_json::Value::Object(map)
    }

    /// Java: `ServerCommandStatus.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ServerCommand::base_from_json(json);
        Self {
            command_nr: base.command_nr,
            server_status: json.get("serverStatus").and_then(|v| v.as_str()).and_then(ServerStatus::from_name),
            message: json.get("message").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        }
    }
}

impl NetCommand for ServerCommandStatus {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerStatus
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let cmd = ServerCommandStatus::new(ServerStatus::FumbblError, "Connected");
        assert_eq!(cmd.get_server_status(), Some(ServerStatus::FumbblError));
        assert_eq!(cmd.get_message(), "Connected");
    }

    #[test]
    fn default_empty() {
        let cmd = ServerCommandStatus::default();
        assert!(cmd.server_status.is_none());
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ServerCommandStatus::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ServerCommandStatus::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ServerCommandStatus::default());
        assert!(s.contains("ServerCommandStatus"));
    }

    #[test]
    fn get_id_is_server_status() {
        assert_eq!(ServerCommandStatus::default().get_id(), NetCommandId::ServerStatus);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_status() {
        let cmd = ServerCommandStatus::new(ServerStatus::ErrorWrongPassword, "bad password");
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "serverStatus");
        assert_eq!(json["serverStatus"], "Wrong Password");
        assert_eq!(json["message"], "bad password");
    }

    #[test]
    fn round_trip_with_status() {
        let mut cmd = ServerCommandStatus::new(ServerStatus::ErrorGameInUse, "in use");
        cmd.command_nr = 4;
        let json = cmd.to_json_value();
        let restored = ServerCommandStatus::from_json(&json);
        assert_eq!(restored.command_nr, 4);
        assert_eq!(restored.server_status, Some(ServerStatus::ErrorGameInUse));
        assert_eq!(restored.message, "in use");
    }

    #[test]
    fn round_trip_with_no_status() {
        let cmd = ServerCommandStatus::default();
        let json = cmd.to_json_value();
        let restored = ServerCommandStatus::from_json(&json);
        assert!(restored.server_status.is_none());
        assert!(restored.message.is_empty());
    }
}
