/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandHandOver`.
/// Sent when a player performs a hand-off to a nearby teammate.
use ffb_model::enums::NetCommandId;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

#[derive(Debug, Clone, Default)]
pub struct ClientCommandHandOver {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `fActingPlayerId`
    pub acting_player_id: Option<String>,
    /// Java: `fCatcherId`
    pub catcher_id: Option<String>,
}

impl ClientCommandHandOver {
    pub fn new() -> Self { Self::default() }

    pub fn with_players(
        acting_player_id: impl Into<String>,
        catcher_id: impl Into<String>,
    ) -> Self {
        Self {
            entropy: None,
            acting_player_id: Some(acting_player_id.into()),
            catcher_id: Some(catcher_id.into()),
        }
    }

    pub fn get_acting_player_id(&self) -> Option<&str> { self.acting_player_id.as_deref() }
    pub fn get_catcher_id(&self) -> Option<&str> { self.catcher_id.as_deref() }

    /// Java: `ClientCommandHandOver.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("actingPlayerId".to_string(), serde_json::json!(self.acting_player_id));
        map.insert("catcherId".to_string(), serde_json::json!(self.catcher_id));
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandHandOver.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            acting_player_id: json.get("actingPlayerId").and_then(|v| v.as_str()).map(String::from),
            catcher_id: json.get("catcherId").and_then(|v| v.as_str()).map(String::from),
        }
    }
}

impl NetCommand for ClientCommandHandOver {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientHandOver
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored_correctly() {
        let cmd = ClientCommandHandOver::with_players("thrower", "catcher");
        assert_eq!(cmd.get_acting_player_id(), Some("thrower"));
        assert_eq!(cmd.get_catcher_id(), Some("catcher"));
    }

    #[test]
    fn default_both_none() {
        let cmd = ClientCommandHandOver::new();
        assert!(cmd.acting_player_id.is_none());
        assert!(cmd.catcher_id.is_none());
    }
#[test]    fn new_constructor_creates_default() { let cmd = ClientCommandHandOver::new(); let _ = cmd; }

    #[test]
    fn catcher_id_stored() {
        let cmd = ClientCommandHandOver::with_players("thrower", "catcher2");
        assert_eq!(cmd.get_catcher_id(), Some("catcher2"));
    }

    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandHandOver::default()).is_empty());
    }

    #[test]
    fn get_id_is_client_hand_over() {
        assert_eq!(ClientCommandHandOver::new().get_id(), NetCommandId::ClientHandOver);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_catcher_id() {
        let cmd = ClientCommandHandOver::with_players("thrower", "catcher");
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientHandOver");
        assert_eq!(json["catcherId"], "catcher");
    }

    #[test]
    fn round_trip_with_players_and_entropy() {
        let mut cmd = ClientCommandHandOver::with_players("thrower", "catcher");
        cmd.entropy = Some(3);
        let json = cmd.to_json_value();
        let restored = ClientCommandHandOver::from_json(&json);
        assert_eq!(restored.entropy, Some(3));
        assert_eq!(restored.get_acting_player_id(), Some("thrower"));
        assert_eq!(restored.get_catcher_id(), Some("catcher"));
    }

    #[test]
    fn round_trip_default() {
        let cmd = ClientCommandHandOver::new();
        let json = cmd.to_json_value();
        let restored = ClientCommandHandOver::from_json(&json);
        assert!(restored.acting_player_id.is_none());
        assert!(restored.catcher_id.is_none());
    }
}
