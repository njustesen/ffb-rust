/// 1:1 translation of com.fumbbl.ffb.net.commands.ClientCommandDebugClientState.
use ffb_model::enums::{ClientStateId, NetCommandId};
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

#[derive(Debug, Clone, Default)]
pub struct ClientCommandDebugClientState {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `fClientStateId`
    pub client_state_id: Option<ClientStateId>,
}

impl ClientCommandDebugClientState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Java: `getClientStateId()`
    pub fn get_client_state_id(&self) -> Option<ClientStateId> {
        self.client_state_id
    }

    /// Java: `ClientCommandDebugClientState.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        if let Some(client_state_id) = self.client_state_id {
            map.insert("clientStateId".to_string(), serde_json::json!(client_state_id.name()));
        }
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandDebugClientState.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            client_state_id: json.get("clientStateId").and_then(|v| v.as_str()).and_then(ClientStateId::from_name),
        }
    }
}

impl NetCommand for ClientCommandDebugClientState {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientDebugClientState
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_no_state_id() {
        let cmd = ClientCommandDebugClientState::new();
        assert!(cmd.get_client_state_id().is_none());
    }

    #[test]
    fn stores_client_state_id() {
        let cmd = ClientCommandDebugClientState {
            entropy: None,
            client_state_id: Some(ClientStateId::Login),
        };
        assert_eq!(cmd.get_client_state_id(), Some(ClientStateId::Login));
    }

    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandDebugClientState::new()).is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandDebugClientState::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandDebugClientState::default());
        assert!(s.contains("ClientCommandDebugClientState"));
    }

    #[test]
    fn get_id_is_client_debug_client_state() {
        assert_eq!(ClientCommandDebugClientState::new().get_id(), NetCommandId::ClientDebugClientState);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_client_state_id() {
        let cmd = ClientCommandDebugClientState {
            entropy: None,
            client_state_id: Some(ClientStateId::Block),
        };
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientDebugClientState");
        assert_eq!(json["clientStateId"], "block");
    }

    #[test]
    fn round_trip_with_state_id_and_entropy() {
        let cmd = ClientCommandDebugClientState {
            entropy: Some(5),
            client_state_id: Some(ClientStateId::Kickoff),
        };
        let json = cmd.to_json_value();
        let restored = ClientCommandDebugClientState::from_json(&json);
        assert_eq!(restored.entropy, Some(5));
        assert_eq!(restored.client_state_id, Some(ClientStateId::Kickoff));
    }

    #[test]
    fn round_trip_with_no_state_id() {
        let cmd = ClientCommandDebugClientState::new();
        let json = cmd.to_json_value();
        let restored = ClientCommandDebugClientState::from_json(&json);
        assert!(restored.client_state_id.is_none());
    }
}
