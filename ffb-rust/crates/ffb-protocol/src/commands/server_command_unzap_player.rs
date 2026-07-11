use ffb_model::enums::NetCommandId;
use crate::commands::server_command::ServerCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandUnzapPlayer`.
/// Instructs the client to restore a previously zapped player.
#[derive(Debug, Clone, Default)]
pub struct ServerCommandUnzapPlayer {
    /// Java: base-class `ServerCommand.fCommandNr`.
    pub command_nr: i32,
    /// Java: `teamId` — the team the player belongs to.
    pub team_id: String,
    /// Java: `playerId` — the player to un-zap.
    pub player_id: String,
}

impl ServerCommandUnzapPlayer {
    pub fn new(team_id: impl Into<String>, player_id: impl Into<String>) -> Self {
        Self { command_nr: 0, team_id: team_id.into(), player_id: player_id.into() }
    }
    pub fn get_team_id(&self) -> &str { &self.team_id }
    pub fn get_player_id(&self) -> &str { &self.player_id }

    /// Java: `ServerCommandUnzapPlayer.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ServerCommand { command_nr: self.command_nr };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("teamId".to_string(), serde_json::json!(self.team_id));
        map.insert("playerId".to_string(), serde_json::json!(self.player_id));
        serde_json::Value::Object(map)
    }

    /// Java: `ServerCommandUnzapPlayer.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ServerCommand::base_from_json(json);
        Self {
            command_nr: base.command_nr,
            team_id: json.get("teamId").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
            player_id: json.get("playerId").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        }
    }
}

impl NetCommand for ServerCommandUnzapPlayer {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerUnzapPlayer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let cmd = ServerCommandUnzapPlayer::new("team1", "p1");
        assert_eq!(cmd.get_team_id(), "team1");
        assert_eq!(cmd.get_player_id(), "p1");
    }

    #[test]
    fn default_same_as_new() {
        let _ = ServerCommandUnzapPlayer::default();
    }

    #[test]
    fn debug_format_works() {
        let v = ServerCommandUnzapPlayer::new("t", "p");
        assert!(!format!("{:?}", v).is_empty());
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ServerCommandUnzapPlayer::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ServerCommandUnzapPlayer::default().clone();
    }

    #[test]
    fn get_id_is_server_unzap_player() {
        assert_eq!(ServerCommandUnzapPlayer::default().get_id(), NetCommandId::ServerUnzapPlayer);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_ids() {
        let cmd = ServerCommandUnzapPlayer::new("team1", "p1");
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "serverUnzapPlayer");
        assert_eq!(json["teamId"], "team1");
        assert_eq!(json["playerId"], "p1");
    }

    #[test]
    fn round_trip_with_ids() {
        let mut cmd = ServerCommandUnzapPlayer::new("team1", "p1");
        cmd.command_nr = 8;
        let json = cmd.to_json_value();
        let restored = ServerCommandUnzapPlayer::from_json(&json);
        assert_eq!(restored.command_nr, 8);
        assert_eq!(restored.team_id, "team1");
        assert_eq!(restored.player_id, "p1");
    }

    #[test]
    fn round_trip_with_default() {
        let cmd = ServerCommandUnzapPlayer::default();
        let json = cmd.to_json_value();
        let restored = ServerCommandUnzapPlayer::from_json(&json);
        assert!(restored.team_id.is_empty());
        assert!(restored.player_id.is_empty());
    }
}
