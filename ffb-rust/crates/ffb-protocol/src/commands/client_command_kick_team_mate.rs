/// 1:1 translation of com.fumbbl.ffb.net.commands.ClientCommandKickTeamMate.
use ffb_model::enums::NetCommandId;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

#[derive(Debug, Clone, Default)]
pub struct ClientCommandKickTeamMate {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `fKickedPlayerId`
    pub kicked_player_id: Option<String>,
    /// Java: `fActingPlayerId`
    pub acting_player_id: Option<String>,
    /// Java: `fNumDice`
    pub num_dice: i32,
}

impl ClientCommandKickTeamMate {
    pub fn new() -> Self {
        Self::default()
    }

    /// Java: `getKickedPlayerId()`
    pub fn get_kicked_player_id(&self) -> Option<&str> {
        self.kicked_player_id.as_deref()
    }

    /// Java: `getActingPlayerId()`
    pub fn get_acting_player_id(&self) -> Option<&str> {
        self.acting_player_id.as_deref()
    }

    /// Java: `getNumDice()`
    pub fn get_num_dice(&self) -> i32 {
        self.num_dice
    }

    /// Java: `ClientCommandKickTeamMate.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("actingPlayerId".to_string(), serde_json::json!(self.acting_player_id));
        map.insert("kickedPlayerId".to_string(), serde_json::json!(self.kicked_player_id));
        map.insert("nrOfDice".to_string(), serde_json::json!(self.num_dice));
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandKickTeamMate.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            acting_player_id: json.get("actingPlayerId").and_then(|v| v.as_str()).map(String::from),
            kicked_player_id: json.get("kickedPlayerId").and_then(|v| v.as_str()).map(String::from),
            num_dice: json.get("nrOfDice").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
        }
    }
}

impl NetCommand for ClientCommandKickTeamMate {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientKickTeamMate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_num_dice_is_zero() {
        let cmd = ClientCommandKickTeamMate::new();
        assert_eq!(cmd.get_num_dice(), 0);
    }

    #[test]
    fn stores_player_ids_and_num_dice() {
        let cmd = ClientCommandKickTeamMate {
            entropy: None,
            kicked_player_id: Some("kicked_1".to_string()),
            acting_player_id: Some("acting_1".to_string()),
            num_dice: 2,
        };
        assert_eq!(cmd.get_kicked_player_id(), Some("kicked_1"));
        assert_eq!(cmd.get_acting_player_id(), Some("acting_1"));
        assert_eq!(cmd.get_num_dice(), 2);
    }

    #[test]
    fn default_ids_are_none() {
        let cmd = ClientCommandKickTeamMate::default();
        assert!(cmd.get_kicked_player_id().is_none());
        assert!(cmd.get_acting_player_id().is_none());
    }


    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandKickTeamMate::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandKickTeamMate::default().clone();
    }

    #[test]
    fn get_id_is_client_kick_team_mate() {
        assert_eq!(ClientCommandKickTeamMate::new().get_id(), NetCommandId::ClientKickTeamMate);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_nr_of_dice() {
        let cmd = ClientCommandKickTeamMate {
            entropy: None,
            kicked_player_id: Some("kicked_1".to_string()),
            acting_player_id: Some("acting_1".to_string()),
            num_dice: 2,
        };
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientKickTeamMate");
        assert_eq!(json["nrOfDice"], 2);
    }

    #[test]
    fn round_trip_with_fields_and_entropy() {
        let cmd = ClientCommandKickTeamMate {
            entropy: Some(7),
            kicked_player_id: Some("kicked_1".to_string()),
            acting_player_id: Some("acting_1".to_string()),
            num_dice: 3,
        };
        let json = cmd.to_json_value();
        let restored = ClientCommandKickTeamMate::from_json(&json);
        assert_eq!(restored.entropy, Some(7));
        assert_eq!(restored.get_kicked_player_id(), Some("kicked_1"));
        assert_eq!(restored.get_acting_player_id(), Some("acting_1"));
        assert_eq!(restored.get_num_dice(), 3);
    }

    #[test]
    fn round_trip_default() {
        let cmd = ClientCommandKickTeamMate::new();
        let json = cmd.to_json_value();
        let restored = ClientCommandKickTeamMate::from_json(&json);
        assert!(restored.kicked_player_id.is_none());
        assert!(restored.acting_player_id.is_none());
        assert_eq!(restored.num_dice, 0);
    }
}
