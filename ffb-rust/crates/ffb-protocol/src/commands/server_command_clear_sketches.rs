use ffb_model::enums::NetCommandId;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandClearSketches`.
/// Instructs the client to clear all sketches from the field view.
#[derive(Debug, Clone, Default)]
pub struct ServerCommandClearSketches {
    /// Java: base-class `ServerCommand.fCommandNr`.
    pub command_nr: i32,
}

impl ServerCommandClearSketches {
    pub fn new() -> Self { Self::default() }

    /// Java: `ServerCommandClearSketches.toJsonValue()`. The Java class only
    /// emits `netCommandId` — `commandNr` is never written for this command.
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({ "netCommandId": self.get_id().name() })
    }

    /// Java: `ServerCommandClearSketches.initFrom(source, jsonValue)`.
    pub fn from_json(_json: &serde_json::Value) -> Self {
        Self::default()
    }
}

impl NetCommand for ServerCommandClearSketches {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerClearSketches
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_be_created() {
        let _ = ServerCommandClearSketches::new();
    }

    #[test]
    fn get_id_is_server_clear_sketches() {
        assert_eq!(ServerCommandClearSketches::new().get_id(), NetCommandId::ServerClearSketches);
    }

    #[test]
    fn to_json_value_has_only_net_command_id() {
        let json = ServerCommandClearSketches::new().to_json_value();
        assert_eq!(json["netCommandId"], "serverClearSketches");
        assert!(json.get("commandNr").is_none());
    }

    #[test]
    fn round_trip() {
        let cmd = ServerCommandClearSketches::new();
        let json = cmd.to_json_value();
        let _restored = ServerCommandClearSketches::from_json(&json);
    }
}
