use ffb_model::enums::NetCommandId;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of ClientCommandTransferReplayControl (Java field: coach).
#[derive(Debug, Clone, Default)]
pub struct ClientCommandTransferReplayControl {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    pub coach: Option<String>,
}

impl ClientCommandTransferReplayControl {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_coach(coach: impl Into<String>) -> Self {
        Self { entropy: None, coach: Some(coach.into()) }
    }

    pub fn get_coach(&self) -> Option<&str> {
        self.coach.as_deref()
    }

    /// Java: `ClientCommandTransferReplayControl.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("coach".to_string(), serde_json::json!(self.coach));
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandTransferReplayControl.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            coach: json.get("coach").and_then(|v| v.as_str()).map(|s| s.to_string()),
        }
    }
}

impl NetCommand for ClientCommandTransferReplayControl {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientTransferReplayControl
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_no_coach() {
        let cmd = ClientCommandTransferReplayControl::new();
        assert!(cmd.get_coach().is_none());
    }

    #[test]
    fn with_coach_stores_value() {
        let cmd = ClientCommandTransferReplayControl::with_coach("coach-abc");
        assert_eq!(cmd.get_coach(), Some("coach-abc"));
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandTransferReplayControl::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandTransferReplayControl::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandTransferReplayControl::default());
        assert!(s.contains("ClientCommandTransferReplayControl"));
    }

    #[test]
    fn get_id_is_client_transfer_replay_control() {
        assert_eq!(ClientCommandTransferReplayControl::new().get_id(), NetCommandId::ClientTransferReplayControl);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_coach() {
        let cmd = ClientCommandTransferReplayControl::with_coach("coach-1");
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientTransferReplayControl");
        assert_eq!(json["coach"], "coach-1");
    }

    #[test]
    fn round_trip_with_coach_and_entropy() {
        let mut cmd = ClientCommandTransferReplayControl::with_coach("coach-2");
        cmd.entropy = Some(6);
        let json = cmd.to_json_value();
        let restored = ClientCommandTransferReplayControl::from_json(&json);
        assert_eq!(restored.entropy, Some(6));
        assert_eq!(restored.get_coach(), Some("coach-2"));
    }

    #[test]
    fn round_trip_with_no_coach() {
        let cmd = ClientCommandTransferReplayControl::new();
        let json = cmd.to_json_value();
        let restored = ClientCommandTransferReplayControl::from_json(&json);
        assert!(restored.get_coach().is_none());
    }
}
