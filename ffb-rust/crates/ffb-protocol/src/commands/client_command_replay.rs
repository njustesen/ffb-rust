use ffb_model::enums::NetCommandId;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandReplay`.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandReplay {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `fGameId`
    pub game_id: i64,
    /// Java: `fReplayToCommandNr`
    pub replay_to_command_nr: i32,
    /// Java: `coach`
    pub coach: Option<String>,
}

impl ClientCommandReplay {
    pub fn new() -> Self { Self::default() }

    pub fn with_params(game_id: i64, replay_to_command_nr: i32, coach: impl Into<String>) -> Self {
        Self {
            entropy: None,
            game_id,
            replay_to_command_nr,
            coach: Some(coach.into()),
        }
    }

    pub fn get_game_id(&self) -> i64 { self.game_id }
    pub fn get_replay_to_command_nr(&self) -> i32 { self.replay_to_command_nr }
    pub fn get_coach(&self) -> Option<&str> { self.coach.as_deref() }

    /// Java: `ClientCommandReplay.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("gameId".to_string(), serde_json::json!(self.game_id));
        map.insert("replayToCommandNr".to_string(), serde_json::json!(self.replay_to_command_nr));
        if let Some(coach) = &self.coach {
            map.insert("coach".to_string(), serde_json::json!(coach));
        }
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandReplay.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            game_id: json.get("gameId").and_then(|v| v.as_i64()).unwrap_or(0),
            replay_to_command_nr: json.get("replayToCommandNr").and_then(|v| v.as_i64()).map(|v| v as i32).unwrap_or(0),
            coach: json.get("coach").and_then(|v| v.as_str()).map(String::from),
        }
    }
}

impl NetCommand for ClientCommandReplay {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientReplay
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let cmd = ClientCommandReplay::with_params(42, 100, "coach1");
        assert_eq!(cmd.get_game_id(), 42);
        assert_eq!(cmd.get_replay_to_command_nr(), 100);
        assert_eq!(cmd.get_coach(), Some("coach1"));
    }

    #[test]
    fn default_is_zeroed() {
        let cmd = ClientCommandReplay::new();
        assert_eq!(cmd.game_id, 0);
        assert_eq!(cmd.replay_to_command_nr, 0);
        assert!(cmd.coach.is_none());
    }

    #[test]
    fn large_game_id_stored() {
        let cmd = ClientCommandReplay::with_params(i64::MAX, 0, "coach");
        assert_eq!(cmd.get_game_id(), i64::MAX);
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandReplay::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandReplay::default().clone();
    }

    #[test]
    fn get_id_is_client_replay() {
        assert_eq!(ClientCommandReplay::new().get_id(), NetCommandId::ClientReplay);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_fields() {
        let cmd = ClientCommandReplay::with_params(7, 3, "coachX");
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientReplay");
        assert_eq!(json["gameId"], 7);
        assert_eq!(json["replayToCommandNr"], 3);
        assert_eq!(json["coach"], "coachX");
    }

    #[test]
    fn round_trip_with_data() {
        let mut cmd = ClientCommandReplay::with_params(99, 12, "coachY");
        cmd.entropy = Some(2);
        let json = cmd.to_json_value();
        let restored = ClientCommandReplay::from_json(&json);
        assert_eq!(restored.entropy, Some(2));
        assert_eq!(restored.get_game_id(), 99);
        assert_eq!(restored.get_replay_to_command_nr(), 12);
        assert_eq!(restored.get_coach(), Some("coachY"));
    }

    #[test]
    fn round_trip_default() {
        let cmd = ClientCommandReplay::default();
        let json = cmd.to_json_value();
        let restored = ClientCommandReplay::from_json(&json);
        assert_eq!(restored.game_id, 0);
        assert_eq!(restored.replay_to_command_nr, 0);
        assert!(restored.coach.is_none());
        assert!(restored.entropy.is_none());
    }
}
