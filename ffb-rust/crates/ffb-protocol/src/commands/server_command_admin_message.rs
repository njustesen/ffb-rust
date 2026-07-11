use ffb_model::enums::NetCommandId;
use crate::commands::server_command::ServerCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandAdminMessage`.
/// Carries one or more admin messages from server to client.
#[derive(Debug, Clone, Default)]
pub struct ServerCommandAdminMessage {
    /// Java: base-class `ServerCommand.fCommandNr`.
    pub command_nr: i32,
    /// Java: `fMessages` — list of admin message strings.
    pub messages: Vec<String>,
}

impl ServerCommandAdminMessage {
    pub fn new(messages: Vec<String>) -> Self { Self { command_nr: 0, messages } }
    pub fn get_messages(&self) -> &[String] { &self.messages }
    pub fn add_message(&mut self, message: impl Into<String>) {
        self.messages.push(message.into());
    }

    /// Java: `isReplayable()` — admin messages are not stored in replays.
    pub fn is_replayable(&self) -> bool {
        false
    }

    /// Java: `ServerCommandAdminMessage.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ServerCommand { command_nr: self.command_nr };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("messageArray".to_string(), serde_json::json!(self.messages));
        serde_json::Value::Object(map)
    }

    /// Java: `ServerCommandAdminMessage.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ServerCommand::base_from_json(json);
        let messages = json
            .get("messageArray")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .filter(|s| !s.is_empty())
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();
        Self { command_nr: base.command_nr, messages }
    }
}

impl NetCommand for ServerCommandAdminMessage {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerAdminMessage
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_messages() {
        let cmd = ServerCommandAdminMessage::new(vec!["hello".into(), "world".into()]);
        assert_eq!(cmd.get_messages(), &["hello", "world"]);
    }

    #[test]
    fn add_message_appends() {
        let mut cmd = ServerCommandAdminMessage::default();
        cmd.add_message("hi");
        assert_eq!(cmd.messages.len(), 1);
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ServerCommandAdminMessage::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ServerCommandAdminMessage::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ServerCommandAdminMessage::default());
        assert!(s.contains("ServerCommandAdminMessage"));
    }

    #[test]
    fn get_id_is_server_admin_message() {
        assert_eq!(ServerCommandAdminMessage::default().get_id(), NetCommandId::ServerAdminMessage);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_message_array() {
        let cmd = ServerCommandAdminMessage::new(vec!["hi".into()]);
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "serverAdminMessage");
        assert_eq!(json["messageArray"][0], "hi");
    }

    #[test]
    fn round_trip_with_messages() {
        let mut cmd = ServerCommandAdminMessage::new(vec!["hello".into(), "world".into()]);
        cmd.command_nr = 4;
        let json = cmd.to_json_value();
        let restored = ServerCommandAdminMessage::from_json(&json);
        assert_eq!(restored.command_nr, 4);
        assert_eq!(restored.messages, vec!["hello".to_string(), "world".to_string()]);
    }

    #[test]
    fn round_trip_with_no_messages() {
        let cmd = ServerCommandAdminMessage::default();
        let json = cmd.to_json_value();
        let restored = ServerCommandAdminMessage::from_json(&json);
        assert!(restored.messages.is_empty());
    }
}
