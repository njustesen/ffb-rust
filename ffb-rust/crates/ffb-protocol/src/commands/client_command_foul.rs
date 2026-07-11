/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandFoul`.
/// Sent when a player initiates a foul action.
use ffb_model::enums::NetCommandId;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

#[derive(Debug, Clone, Default)]
pub struct ClientCommandFoul {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `fActingPlayerId`
    pub acting_player_id: Option<String>,
    /// Java: `fDefenderId`
    pub defender_id: Option<String>,
    /// Java: `usingChainsaw`
    pub using_chainsaw: bool,
}

impl ClientCommandFoul {
    pub fn new() -> Self { Self::default() }

    pub fn with_players(
        acting_player_id: impl Into<String>,
        defender_id: impl Into<String>,
        using_chainsaw: bool,
    ) -> Self {
        Self {
            entropy: None,
            acting_player_id: Some(acting_player_id.into()),
            defender_id: Some(defender_id.into()),
            using_chainsaw,
        }
    }

    pub fn get_acting_player_id(&self) -> Option<&str> { self.acting_player_id.as_deref() }
    pub fn get_defender_id(&self) -> Option<&str> { self.defender_id.as_deref() }
    pub fn is_using_chainsaw(&self) -> bool { self.using_chainsaw }

    /// Java: `ClientCommandFoul.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("actingPlayerId".to_string(), serde_json::json!(self.acting_player_id));
        map.insert("defenderId".to_string(), serde_json::json!(self.defender_id));
        map.insert("usingChainsaw".to_string(), serde_json::json!(self.using_chainsaw));
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandFoul.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            acting_player_id: json.get("actingPlayerId").and_then(|v| v.as_str()).map(String::from),
            defender_id: json.get("defenderId").and_then(|v| v.as_str()).map(String::from),
            using_chainsaw: json.get("usingChainsaw").and_then(|v| v.as_bool()).unwrap_or(false),
        }
    }
}

impl NetCommand for ClientCommandFoul {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientFoul
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored_correctly() {
        let cmd = ClientCommandFoul::with_players("atk", "def", true);
        assert_eq!(cmd.get_acting_player_id(), Some("atk"));
        assert_eq!(cmd.get_defender_id(), Some("def"));
        assert!(cmd.is_using_chainsaw());
    }

    #[test]
    fn default_all_none_and_false() {
        let cmd = ClientCommandFoul::new();
        assert!(cmd.acting_player_id.is_none());
        assert!(cmd.defender_id.is_none());
        assert!(!cmd.using_chainsaw);
    }

    #[test]
    fn no_chainsaw_flag() {
        let cmd = ClientCommandFoul::with_players("a", "b", false);
        assert!(!cmd.is_using_chainsaw());
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandFoul::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandFoul::default().clone();
    }

    #[test]
    fn get_id_is_client_foul() {
        assert_eq!(ClientCommandFoul::new().get_id(), NetCommandId::ClientFoul);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_using_chainsaw() {
        let cmd = ClientCommandFoul::with_players("atk", "def", true);
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientFoul");
        assert_eq!(json["usingChainsaw"], true);
        assert_eq!(json["actingPlayerId"], "atk");
    }

    #[test]
    fn round_trip_with_players_and_entropy() {
        let mut cmd = ClientCommandFoul::with_players("atk", "def", true);
        cmd.entropy = Some(6);
        let json = cmd.to_json_value();
        let restored = ClientCommandFoul::from_json(&json);
        assert_eq!(restored.entropy, Some(6));
        assert_eq!(restored.get_acting_player_id(), Some("atk"));
        assert_eq!(restored.get_defender_id(), Some("def"));
        assert!(restored.is_using_chainsaw());
    }

    #[test]
    fn round_trip_default() {
        let cmd = ClientCommandFoul::new();
        let json = cmd.to_json_value();
        let restored = ClientCommandFoul::from_json(&json);
        assert!(restored.acting_player_id.is_none());
        assert!(restored.defender_id.is_none());
        assert!(!restored.using_chainsaw);
    }
}
