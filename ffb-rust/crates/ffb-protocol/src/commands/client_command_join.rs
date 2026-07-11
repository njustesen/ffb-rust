/// 1:1 translation of com.fumbbl.ffb.net.commands.ClientCommandJoin.
use ffb_model::enums::NetCommandId;
use ffb_model::model::ClientMode;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

#[derive(Debug, Clone, Default)]
pub struct ClientCommandJoin {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `fCoach`
    pub coach: Option<String>,
    /// Java: `fPassword`
    pub password: Option<String>,
    /// Java: `fGameName`
    pub game_name: Option<String>,
    /// Java: `fTeamId`
    pub team_id: Option<String>,
    /// Java: `fTeamName`
    pub team_name: Option<String>,
    /// Java: `fGameId`
    pub game_id: i64,
    /// Java: `fClientMode`
    pub client_mode: Option<ClientMode>,
}

impl ClientCommandJoin {
    pub fn new() -> Self {
        Self::default()
    }

    /// Java: `getCoach()`
    pub fn get_coach(&self) -> Option<&str> {
        self.coach.as_deref()
    }

    /// Java: `getPassword()`
    pub fn get_password(&self) -> Option<&str> {
        self.password.as_deref()
    }

    /// Java: `getGameName()`
    pub fn get_game_name(&self) -> Option<&str> {
        self.game_name.as_deref()
    }

    /// Java: `getTeamId()`
    pub fn get_team_id(&self) -> Option<&str> {
        self.team_id.as_deref()
    }

    /// Java: `getTeamName()`
    pub fn get_team_name(&self) -> Option<&str> {
        self.team_name.as_deref()
    }

    /// Java: `getGameId()`
    pub fn get_game_id(&self) -> i64 {
        self.game_id
    }

    /// Java: `getClientMode()`
    pub fn get_client_mode(&self) -> Option<&ClientMode> {
        self.client_mode.as_ref()
    }

    /// Java: `ClientCommandJoin.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        if let Some(client_mode) = self.client_mode {
            map.insert("clientMode".to_string(), serde_json::json!(client_mode.get_name()));
        }
        map.insert("coach".to_string(), serde_json::json!(self.coach));
        map.insert("password".to_string(), serde_json::json!(self.password));
        map.insert("gameId".to_string(), serde_json::json!(self.game_id));
        map.insert("gameName".to_string(), serde_json::json!(self.game_name));
        map.insert("teamId".to_string(), serde_json::json!(self.team_id));
        map.insert("teamName".to_string(), serde_json::json!(self.team_name));
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandJoin.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            client_mode: json.get("clientMode").and_then(|v| v.as_str()).and_then(ClientMode::for_name),
            coach: json.get("coach").and_then(|v| v.as_str()).map(String::from),
            password: json.get("password").and_then(|v| v.as_str()).map(String::from),
            game_id: json.get("gameId").and_then(|v| v.as_i64()).unwrap_or(0),
            game_name: json.get("gameName").and_then(|v| v.as_str()).map(String::from),
            team_id: json.get("teamId").and_then(|v| v.as_str()).map(String::from),
            team_name: json.get("teamName").and_then(|v| v.as_str()).map(String::from),
        }
    }
}

impl NetCommand for ClientCommandJoin {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientJoin
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_game_id_is_zero() {
        let cmd = ClientCommandJoin::new();
        assert_eq!(cmd.get_game_id(), 0);
    }

    #[test]
    fn stores_coach_and_game_id() {
        let cmd = ClientCommandJoin {
            coach: Some("TestCoach".to_string()),
            game_id: 42,
            ..Default::default()
        };
        assert_eq!(cmd.get_coach(), Some("TestCoach"));
        assert_eq!(cmd.get_game_id(), 42);
    }

    #[test]
    fn team_id_stored() {
        let cmd = ClientCommandJoin {
            team_id: Some("team-abc".to_string()),
            ..Default::default()
        };
        assert_eq!(cmd.get_team_id(), Some("team-abc"));
    }


    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandJoin::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandJoin::default().clone();
    }

    #[test]
    fn get_id_is_client_join() {
        assert_eq!(ClientCommandJoin::new().get_id(), NetCommandId::ClientJoin);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_client_mode() {
        let cmd = ClientCommandJoin {
            client_mode: Some(ClientMode::PLAYER),
            ..Default::default()
        };
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientJoin");
        assert_eq!(json["clientMode"], "player");
    }

    #[test]
    fn round_trip_with_all_fields_and_entropy() {
        let cmd = ClientCommandJoin {
            entropy: Some(1),
            coach: Some("Coach".to_string()),
            password: Some("pw".to_string()),
            game_name: Some("Game1".to_string()),
            team_id: Some("t1".to_string()),
            team_name: Some("Team1".to_string()),
            game_id: 77,
            client_mode: Some(ClientMode::SPECTATOR),
        };
        let json = cmd.to_json_value();
        let restored = ClientCommandJoin::from_json(&json);
        assert_eq!(restored.entropy, Some(1));
        assert_eq!(restored.get_coach(), Some("Coach"));
        assert_eq!(restored.get_password(), Some("pw"));
        assert_eq!(restored.get_game_name(), Some("Game1"));
        assert_eq!(restored.get_team_id(), Some("t1"));
        assert_eq!(restored.get_team_name(), Some("Team1"));
        assert_eq!(restored.get_game_id(), 77);
        assert_eq!(restored.get_client_mode(), Some(&ClientMode::SPECTATOR));
    }

    #[test]
    fn round_trip_default() {
        let cmd = ClientCommandJoin::default();
        let json = cmd.to_json_value();
        let restored = ClientCommandJoin::from_json(&json);
        assert!(restored.coach.is_none());
        assert!(restored.client_mode.is_none());
        assert_eq!(restored.game_id, 0);
    }
}
