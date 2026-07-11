use ffb_model::enums::NetCommandId;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandUseTeamMatesWisdom`.
/// Sent when a BB2025 player uses Team Mates Wisdom (no payload).
#[derive(Debug, Clone, Default)]
pub struct ClientCommandUseTeamMatesWisdom {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
}

impl ClientCommandUseTeamMatesWisdom {
    pub fn new() -> Self { Self::default() }

    /// Java: `ClientCommandUseTeamMatesWisdom.toJsonValue()` (inherited from `ClientCommand`, no override).
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let map = base.base_json_fields(self.get_id());
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandUseTeamMatesWisdom.initFrom(source, jsonValue)` (inherited from `ClientCommand`).
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self { entropy: base.entropy }
    }
}

impl NetCommand for ClientCommandUseTeamMatesWisdom {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientUseTeamMatesWisdom
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn can_construct() { let _ = ClientCommandUseTeamMatesWisdom::new(); }
    #[test]
    fn default_works() { let _ = ClientCommandUseTeamMatesWisdom::default(); }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandUseTeamMatesWisdom::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandUseTeamMatesWisdom::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandUseTeamMatesWisdom::default());
        assert!(s.contains("ClientCommandUseTeamMatesWisdom"));
    }

    #[test]
    fn get_id_is_client_use_team_mates_wisdom() {
        assert_eq!(ClientCommandUseTeamMatesWisdom::new().get_id(), NetCommandId::ClientUseTeamMatesWisdom);
    }

    #[test]
    fn to_json_value_has_net_command_id() {
        let json = ClientCommandUseTeamMatesWisdom::new().to_json_value();
        assert_eq!(json["netCommandId"], "clientUseTeamMatesWisdom");
    }

    #[test]
    fn round_trip_with_entropy() {
        let mut cmd = ClientCommandUseTeamMatesWisdom::new();
        cmd.entropy = Some(2);
        let json = cmd.to_json_value();
        let restored = ClientCommandUseTeamMatesWisdom::from_json(&json);
        assert_eq!(restored.entropy, Some(2));
    }

    #[test]
    fn round_trip_with_no_entropy() {
        let cmd = ClientCommandUseTeamMatesWisdom::new();
        let json = cmd.to_json_value();
        let restored = ClientCommandUseTeamMatesWisdom::from_json(&json);
        assert!(restored.entropy.is_none());
    }
}
