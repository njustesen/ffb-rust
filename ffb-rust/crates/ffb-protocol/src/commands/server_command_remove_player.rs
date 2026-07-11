use ffb_model::enums::NetCommandId;
use crate::commands::server_command::ServerCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandRemovePlayer`.
/// Instructs the client to remove a player from the game view.
#[derive(Debug, Clone, Default)]
pub struct ServerCommandRemovePlayer {
    /// Java: base-class `ServerCommand.fCommandNr`.
    pub command_nr: i32,
    /// Java: `fPlayerId` — the player to remove.
    pub player_id: String,
}

impl ServerCommandRemovePlayer {
    pub fn new(player_id: impl Into<String>) -> Self {
        Self { command_nr: 0, player_id: player_id.into() }
    }
    pub fn get_player_id(&self) -> &str { &self.player_id }

    /// Java: `ServerCommandRemovePlayer.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ServerCommand { command_nr: self.command_nr };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("playerId".to_string(), serde_json::json!(self.player_id));
        serde_json::Value::Object(map)
    }

    /// Java: `ServerCommandRemovePlayer.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ServerCommand::base_from_json(json);
        Self {
            command_nr: base.command_nr,
            player_id: json.get("playerId").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        }
    }
}

impl NetCommand for ServerCommandRemovePlayer {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerRemovePlayer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_id_stored() {
        let cmd = ServerCommandRemovePlayer::new("p1");
        assert_eq!(cmd.get_player_id(), "p1");
    }

    #[test]
    fn default_empty() {
        let cmd = ServerCommandRemovePlayer::default();
        assert!(cmd.player_id.is_empty());
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ServerCommandRemovePlayer::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ServerCommandRemovePlayer::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ServerCommandRemovePlayer::default());
        assert!(s.contains("ServerCommandRemovePlayer"));
    }

    #[test]
    fn get_id_is_server_remove_player() {
        assert_eq!(ServerCommandRemovePlayer::new("p1").get_id(), NetCommandId::ServerRemovePlayer);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_player_id() {
        let mut cmd = ServerCommandRemovePlayer::new("p9");
        cmd.command_nr = 3;
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "serverRemovePlayer");
        assert_eq!(json["commandNr"], 3);
        assert_eq!(json["playerId"], "p9");
    }

    #[test]
    fn round_trip_with_data() {
        let mut cmd = ServerCommandRemovePlayer::new("p42");
        cmd.command_nr = 7;
        let json = cmd.to_json_value();
        let restored = ServerCommandRemovePlayer::from_json(&json);
        assert_eq!(restored.command_nr, 7);
        assert_eq!(restored.player_id, "p42");
    }

    #[test]
    fn round_trip_with_default() {
        let cmd = ServerCommandRemovePlayer::default();
        let json = cmd.to_json_value();
        let restored = ServerCommandRemovePlayer::from_json(&json);
        assert_eq!(restored.command_nr, 0);
        assert!(restored.player_id.is_empty());
    }
}
