use ffb_model::enums::NetCommandId;
use ffb_model::model::factory_type::FactoryContext;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of ClientCommandStartGame (Java: no fields).
#[derive(Debug, Clone, Default)]
pub struct ClientCommandStartGame {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
}

impl ClientCommandStartGame {
    pub fn new() -> Self {
        Self::default()
    }

    /// Java: `ClientCommandStartGame.toJsonValue()` (calls `super.toJsonValue()` only).
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let map = base.base_json_fields(self.get_id());
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandStartGame.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self { entropy: base.entropy }
    }
}

impl NetCommand for ClientCommandStartGame {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientStartGame
    }

    /// Java: `ClientCommandStartGame.getContext()` overrides to `FactoryContext.APPLICATION`.
    fn get_context(&self) -> FactoryContext {
        FactoryContext::APPLICATION
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_construct() {
        let _cmd = ClientCommandStartGame::new();
    }

    #[test]
    fn default_works() {
        let _cmd = ClientCommandStartGame::default();
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandStartGame::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandStartGame::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandStartGame::default());
        assert!(s.contains("ClientCommandStartGame"));
    }

    #[test]
    fn get_id_is_client_start_game() {
        assert_eq!(ClientCommandStartGame::new().get_id(), NetCommandId::ClientStartGame);
    }

    #[test]
    fn get_context_is_application() {
        assert_eq!(ClientCommandStartGame::new().get_context(), FactoryContext::APPLICATION);
    }

    #[test]
    fn to_json_value_has_net_command_id() {
        let cmd = ClientCommandStartGame::new();
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientStartGame");
    }

    #[test]
    fn round_trip_with_entropy() {
        let mut cmd = ClientCommandStartGame::new();
        cmd.entropy = Some(5);
        let json = cmd.to_json_value();
        let restored = ClientCommandStartGame::from_json(&json);
        assert_eq!(restored.entropy, Some(5));
    }

    #[test]
    fn round_trip_with_no_entropy() {
        let cmd = ClientCommandStartGame::new();
        let json = cmd.to_json_value();
        let restored = ClientCommandStartGame::from_json(&json);
        assert!(restored.entropy.is_none());
    }
}
