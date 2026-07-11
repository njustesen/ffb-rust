use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;
use ffb_model::enums::NetCommandId;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandClearSketches`.
/// Sent when a client clears all field sketches (no payload beyond the
/// `ClientSketchCommand` base — entropy only).
#[derive(Debug, Clone, Default)]
pub struct ClientCommandClearSketches {
    /// Java: base-class `ClientCommand.fEntropy` (via `ClientSketchCommand`).
    pub entropy: Option<u8>,
}

impl ClientCommandClearSketches {
    pub fn new() -> Self { Self::default() }

    /// Java: `ClientCommandClearSketches.requiresControl()` (override) — always `true`.
    pub fn requires_control(&self) -> bool {
        true
    }

    /// Java: `ClientCommandClearSketches.toJsonValue()` — just `super.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let map = base.base_json_fields(self.get_id());
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandClearSketches.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self { entropy: base.entropy }
    }
}

impl NetCommand for ClientCommandClearSketches {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientClearSketches
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn can_construct() { let _ = ClientCommandClearSketches::new(); }

    #[test]
    fn default_same_as_new() { let _ = ClientCommandClearSketches::default(); }

    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandClearSketches::new()).is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandClearSketches::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandClearSketches::default());
        assert!(s.contains("ClientCommandClearSketches"));
    }

    #[test]
    fn requires_control_is_true() {
        assert!(ClientCommandClearSketches::new().requires_control());
    }

    #[test]
    fn get_id_is_client_clear_sketches() {
        assert_eq!(ClientCommandClearSketches::new().get_id(), NetCommandId::ClientClearSketches);
    }

    #[test]
    fn to_json_value_has_net_command_id() {
        let json = ClientCommandClearSketches::new().to_json_value();
        assert_eq!(json["netCommandId"], "clientClearSketches");
    }

    #[test]
    fn round_trip_with_entropy() {
        let mut cmd = ClientCommandClearSketches::new();
        cmd.entropy = Some(2);
        let json = cmd.to_json_value();
        let restored = ClientCommandClearSketches::from_json(&json);
        assert_eq!(restored.entropy, Some(2));
    }

    #[test]
    fn round_trip_with_no_entropy() {
        let cmd = ClientCommandClearSketches::default();
        let json = cmd.to_json_value();
        let restored = ClientCommandClearSketches::from_json(&json);
        assert_eq!(restored.entropy, None);
    }
}
