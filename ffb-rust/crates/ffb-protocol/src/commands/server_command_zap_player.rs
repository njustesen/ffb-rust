use ffb_model::enums::NetCommandId;
use crate::commands::server_command::ServerCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandZapPlayer`.
/// Instructs the client to visually remove (zap) a player from the field.
#[derive(Debug, Clone, Default)]
pub struct ServerCommandZapPlayer {
    /// Java: base-class `ServerCommand.fCommandNr`.
    pub command_nr: i32,
    /// Java: `teamId` — the team the player belongs to.
    pub team_id: String,
    /// Java: `playerId` — the player to zap.
    pub player_id: String,
}

impl ServerCommandZapPlayer {
    pub fn new(team_id: impl Into<String>, player_id: impl Into<String>) -> Self {
        Self { command_nr: 0, team_id: team_id.into(), player_id: player_id.into() }
    }
    pub fn get_team_id(&self) -> &str { &self.team_id }
    pub fn get_player_id(&self) -> &str { &self.player_id }

    /// Java: `ServerCommandZapPlayer.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ServerCommand { command_nr: self.command_nr };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("teamId".to_string(), serde_json::json!(self.team_id));
        map.insert("playerId".to_string(), serde_json::json!(self.player_id));
        serde_json::Value::Object(map)
    }

    /// Java: `ServerCommandZapPlayer.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ServerCommand::base_from_json(json);
        Self {
            command_nr: base.command_nr,
            team_id: json.get("teamId").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
            player_id: json.get("playerId").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        }
    }
}

impl NetCommand for ServerCommandZapPlayer {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerZapPlayer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let cmd = ServerCommandZapPlayer::new("team2", "p5");
        assert_eq!(cmd.get_team_id(), "team2");
        assert_eq!(cmd.get_player_id(), "p5");
    }

    #[test]
    fn default_empty() {
        let cmd = ServerCommandZapPlayer::default();
        assert!(cmd.player_id.is_empty());
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ServerCommandZapPlayer::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ServerCommandZapPlayer::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ServerCommandZapPlayer::default());
        assert!(s.contains("ServerCommandZapPlayer"));
    }

    #[test]
    fn get_id_is_server_zap_player() {
        assert_eq!(ServerCommandZapPlayer::default().get_id(), NetCommandId::ServerZapPlayer);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_ids() {
        let cmd = ServerCommandZapPlayer::new("team2", "p5");
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "serverZapPlayer");
        assert_eq!(json["teamId"], "team2");
        assert_eq!(json["playerId"], "p5");
    }

    #[test]
    fn round_trip_with_ids() {
        let mut cmd = ServerCommandZapPlayer::new("team2", "p5");
        cmd.command_nr = 9;
        let json = cmd.to_json_value();
        let restored = ServerCommandZapPlayer::from_json(&json);
        assert_eq!(restored.command_nr, 9);
        assert_eq!(restored.team_id, "team2");
        assert_eq!(restored.player_id, "p5");
    }

    #[test]
    fn round_trip_with_default() {
        let cmd = ServerCommandZapPlayer::default();
        let json = cmd.to_json_value();
        let restored = ServerCommandZapPlayer::from_json(&json);
        assert!(restored.team_id.is_empty());
        assert!(restored.player_id.is_empty());
    }
}
