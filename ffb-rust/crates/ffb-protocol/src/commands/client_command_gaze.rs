/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandGaze`.
/// Sent when a player uses Hypnotic Gaze on a victim.
use ffb_model::enums::NetCommandId;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

#[derive(Debug, Clone, Default)]
pub struct ClientCommandGaze {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `fActingPlayerId`
    pub acting_player_id: Option<String>,
    /// Java: `fVictimId`
    pub victim_id: Option<String>,
}

impl ClientCommandGaze {
    pub fn new() -> Self { Self::default() }

    pub fn with_players(
        acting_player_id: impl Into<String>,
        victim_id: impl Into<String>,
    ) -> Self {
        Self {
            entropy: None,
            acting_player_id: Some(acting_player_id.into()),
            victim_id: Some(victim_id.into()),
        }
    }

    pub fn get_acting_player_id(&self) -> Option<&str> { self.acting_player_id.as_deref() }
    pub fn get_victim_id(&self) -> Option<&str> { self.victim_id.as_deref() }

    /// Java: `ClientCommandGaze.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("actingPlayerId".to_string(), serde_json::json!(self.acting_player_id));
        map.insert("victimId".to_string(), serde_json::json!(self.victim_id));
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandGaze.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            acting_player_id: json.get("actingPlayerId").and_then(|v| v.as_str()).map(String::from),
            victim_id: json.get("victimId").and_then(|v| v.as_str()).map(String::from),
        }
    }
}

impl NetCommand for ClientCommandGaze {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientGaze
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored_correctly() {
        let cmd = ClientCommandGaze::with_players("gazer", "victim");
        assert_eq!(cmd.get_acting_player_id(), Some("gazer"));
        assert_eq!(cmd.get_victim_id(), Some("victim"));
    }

    #[test]
    fn default_both_none() {
        let cmd = ClientCommandGaze::new();
        assert!(cmd.acting_player_id.is_none());
        assert!(cmd.victim_id.is_none());
    }
#[test]    fn new_constructor_creates_default() { let cmd = ClientCommandGaze::new(); let _ = cmd; }

    #[test]
    fn victim_id_stored() {
        let cmd = ClientCommandGaze::with_players("g", "v");
        assert_eq!(cmd.get_victim_id(), Some("v"));
    }

    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandGaze::default()).is_empty());
    }

    #[test]
    fn get_id_is_client_gaze() {
        assert_eq!(ClientCommandGaze::new().get_id(), NetCommandId::ClientGaze);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_victim_id() {
        let cmd = ClientCommandGaze::with_players("g", "v");
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientGaze");
        assert_eq!(json["victimId"], "v");
    }

    #[test]
    fn round_trip_with_players_and_entropy() {
        let mut cmd = ClientCommandGaze::with_players("gazer", "victim");
        cmd.entropy = Some(4);
        let json = cmd.to_json_value();
        let restored = ClientCommandGaze::from_json(&json);
        assert_eq!(restored.entropy, Some(4));
        assert_eq!(restored.get_acting_player_id(), Some("gazer"));
        assert_eq!(restored.get_victim_id(), Some("victim"));
    }

    #[test]
    fn round_trip_default() {
        let cmd = ClientCommandGaze::new();
        let json = cmd.to_json_value();
        let restored = ClientCommandGaze::from_json(&json);
        assert!(restored.acting_player_id.is_none());
        assert!(restored.victim_id.is_none());
    }
}
