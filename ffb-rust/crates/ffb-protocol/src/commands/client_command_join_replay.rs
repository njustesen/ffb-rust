/// 1:1 translation of com.fumbbl.ffb.net.commands.ClientCommandJoinReplay.
use ffb_model::enums::NetCommandId;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

#[derive(Debug, Clone, Default)]
pub struct ClientCommandJoinReplay {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `replayName`
    pub replay_name: Option<String>,
    /// Java: `coach`
    pub coach: Option<String>,
    /// Java: `gameId`
    pub game_id: i64,
}

impl ClientCommandJoinReplay {
    pub fn new() -> Self {
        Self::default()
    }

    /// Java: `getReplayName()`
    pub fn get_replay_name(&self) -> Option<&str> {
        self.replay_name.as_deref()
    }

    /// Java: `getCoach()`
    pub fn get_coach(&self) -> Option<&str> {
        self.coach.as_deref()
    }

    /// Java: `getGameId()`
    pub fn get_game_id(&self) -> i64 {
        self.game_id
    }

    /// Java: `ClientCommandJoinReplay.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("name".to_string(), serde_json::json!(self.replay_name));
        map.insert("coach".to_string(), serde_json::json!(self.coach));
        map.insert("gameId".to_string(), serde_json::json!(self.game_id));
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandJoinReplay.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            replay_name: json.get("name").and_then(|v| v.as_str()).map(String::from),
            coach: json.get("coach").and_then(|v| v.as_str()).map(String::from),
            game_id: json.get("gameId").and_then(|v| v.as_i64()).unwrap_or(0),
        }
    }
}

impl NetCommand for ClientCommandJoinReplay {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientJoinReplay
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_game_id_is_zero() {
        let cmd = ClientCommandJoinReplay::new();
        assert_eq!(cmd.get_game_id(), 0);
    }

    #[test]
    fn stores_replay_name_and_coach() {
        let cmd = ClientCommandJoinReplay {
            entropy: None,
            replay_name: Some("replay_001".to_string()),
            coach: Some("CoachA".to_string()),
            game_id: 99,
        };
        assert_eq!(cmd.get_replay_name(), Some("replay_001"));
        assert_eq!(cmd.get_coach(), Some("CoachA"));
        assert_eq!(cmd.get_game_id(), 99);
    }

    #[test]
    fn replay_name_none_by_default() {
        let cmd = ClientCommandJoinReplay::default();
        assert!(cmd.get_replay_name().is_none());
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandJoinReplay::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandJoinReplay::default().clone();
    }

    #[test]
    fn get_id_is_client_join_replay() {
        assert_eq!(ClientCommandJoinReplay::new().get_id(), NetCommandId::ClientJoinReplay);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_name() {
        let cmd = ClientCommandJoinReplay {
            entropy: None,
            replay_name: Some("replay_001".to_string()),
            coach: None,
            game_id: 5,
        };
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientJoinReplay");
        assert_eq!(json["name"], "replay_001");
    }

    #[test]
    fn round_trip_with_fields_and_entropy() {
        let cmd = ClientCommandJoinReplay {
            entropy: Some(9),
            replay_name: Some("replay_001".to_string()),
            coach: Some("CoachA".to_string()),
            game_id: 99,
        };
        let json = cmd.to_json_value();
        let restored = ClientCommandJoinReplay::from_json(&json);
        assert_eq!(restored.entropy, Some(9));
        assert_eq!(restored.get_replay_name(), Some("replay_001"));
        assert_eq!(restored.get_coach(), Some("CoachA"));
        assert_eq!(restored.get_game_id(), 99);
    }

    #[test]
    fn round_trip_default() {
        let cmd = ClientCommandJoinReplay::default();
        let json = cmd.to_json_value();
        let restored = ClientCommandJoinReplay::from_json(&json);
        assert!(restored.replay_name.is_none());
        assert_eq!(restored.game_id, 0);
    }
}
