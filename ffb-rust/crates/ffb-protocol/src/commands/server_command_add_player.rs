use ffb_model::model::player_state::PlayerState;
use ffb_model::model::roster_player::RosterPlayer;
use ffb_model::model::send_to_box_reason::SendToBoxReason;
use ffb_model::enums::NetCommandId;
use crate::commands::server_command::ServerCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandAddPlayer`.
/// Adds a player to the client's view of the game.
#[derive(Debug, Clone, Default)]
pub struct ServerCommandAddPlayer {
    /// Java: base-class `ServerCommand.fCommandNr`.
    pub command_nr: i32,
    /// Java: `fTeamId` — team this player belongs to.
    pub team_id: String,
    /// Java: `fPlayer` — the roster player being added.
    pub player: RosterPlayer,
    /// Java: `fPlayerState` — initial player state.
    pub player_state: PlayerState,
    /// Java: `fSendToBoxReason` — reason for box placement (if any).
    pub send_to_box_reason: Option<SendToBoxReason>,
    /// Java: `fSendToBoxTurn` — turn number when sent to box.
    pub send_to_box_turn: i32,
    /// Java: `fSendToBoxHalf` — half number when sent to box.
    pub send_to_box_half: i32,
}

impl ServerCommandAddPlayer {
    pub fn new(
        team_id: impl Into<String>,
        player: RosterPlayer,
        player_state: PlayerState,
        send_to_box_reason: Option<SendToBoxReason>,
        send_to_box_turn: i32,
    ) -> Self {
        Self {
            command_nr: 0,
            team_id: team_id.into(),
            player,
            player_state,
            send_to_box_reason,
            send_to_box_turn,
            send_to_box_half: 0,
        }
    }
    pub fn get_team_id(&self) -> &str { &self.team_id }
    pub fn get_player(&self) -> &RosterPlayer { &self.player }
    pub fn get_player_state(&self) -> &PlayerState { &self.player_state }
    pub fn get_send_to_box_reason(&self) -> Option<SendToBoxReason> { self.send_to_box_reason }
    pub fn get_send_to_box_turn(&self) -> i32 { self.send_to_box_turn }
    pub fn get_send_to_box_half(&self) -> i32 { self.send_to_box_half }

    /// Java: `ServerCommandAddPlayer.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ServerCommand { command_nr: self.command_nr };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("teamId".to_string(), serde_json::json!(self.team_id));
        // Java's RosterPlayer.toJsonValue() has no Rust-side equivalent yet;
        // fall back to the struct's own serde derive so no field is dropped.
        map.insert("player".to_string(), serde_json::to_value(&self.player).unwrap_or(serde_json::Value::Null));
        map.insert("playerState".to_string(), serde_json::to_value(self.player_state).unwrap_or(serde_json::Value::Null));
        if let Some(reason) = self.send_to_box_reason {
            map.insert("sendToBoxReason".to_string(), serde_json::json!(reason.get_name()));
        }
        map.insert("sendToBoxTurn".to_string(), serde_json::json!(self.send_to_box_turn));
        map.insert("sendToBoxHalf".to_string(), serde_json::json!(self.send_to_box_half));
        serde_json::Value::Object(map)
    }

    /// Java: `ServerCommandAddPlayer.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ServerCommand::base_from_json(json);
        Self {
            command_nr: base.command_nr,
            team_id: json.get("teamId").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
            player: json.get("player").and_then(|v| serde_json::from_value(v.clone()).ok()).unwrap_or_default(),
            player_state: json.get("playerState").and_then(|v| serde_json::from_value(v.clone()).ok()).unwrap_or_default(),
            send_to_box_reason: json.get("sendToBoxReason").and_then(|v| v.as_str()).and_then(SendToBoxReason::for_name),
            send_to_box_turn: json.get("sendToBoxTurn").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            send_to_box_half: json.get("sendToBoxHalf").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
        }
    }
}

impl NetCommand for ServerCommandAddPlayer {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerAddPlayer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let cmd = ServerCommandAddPlayer::new(
            "team1",
            RosterPlayer::default(),
            PlayerState::default(),
            Some(SendToBoxReason::FOUL_BAN),
            3,
        );
        assert_eq!(cmd.get_team_id(), "team1");
        assert_eq!(cmd.get_send_to_box_reason(), Some(SendToBoxReason::FOUL_BAN));
        assert_eq!(cmd.get_send_to_box_turn(), 3);
    }

    #[test]
    fn default_no_box() {
        let cmd = ServerCommandAddPlayer::default();
        assert!(cmd.team_id.is_empty());
        assert!(cmd.send_to_box_reason.is_none());
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ServerCommandAddPlayer::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ServerCommandAddPlayer::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ServerCommandAddPlayer::default());
        assert!(s.contains("ServerCommandAddPlayer"));
    }

    #[test]
    fn get_id_is_server_add_player() {
        assert_eq!(ServerCommandAddPlayer::default().get_id(), NetCommandId::ServerAddPlayer);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_team_id() {
        let mut cmd = ServerCommandAddPlayer::new("team1", RosterPlayer::default(), PlayerState::default(), None, 0);
        cmd.command_nr = 3;
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "serverAddPlayer");
        assert_eq!(json["commandNr"], 3);
        assert_eq!(json["teamId"], "team1");
        assert!(json.get("sendToBoxReason").is_none());
    }

    #[test]
    fn round_trip_with_send_to_box() {
        let mut cmd = ServerCommandAddPlayer::new(
            "team1",
            RosterPlayer::default(),
            PlayerState::default(),
            Some(SendToBoxReason::FOULED),
            2,
        );
        cmd.command_nr = 5;
        cmd.send_to_box_half = 1;
        let json = cmd.to_json_value();
        let restored = ServerCommandAddPlayer::from_json(&json);
        assert_eq!(restored.command_nr, 5);
        assert_eq!(restored.team_id, "team1");
        assert_eq!(restored.send_to_box_reason, Some(SendToBoxReason::FOULED));
        assert_eq!(restored.send_to_box_turn, 2);
        assert_eq!(restored.send_to_box_half, 1);
    }

    #[test]
    fn round_trip_defaults() {
        let cmd = ServerCommandAddPlayer::default();
        let json = cmd.to_json_value();
        let restored = ServerCommandAddPlayer::from_json(&json);
        assert!(restored.team_id.is_empty());
        assert!(restored.send_to_box_reason.is_none());
        assert_eq!(restored.send_to_box_turn, 0);
        assert_eq!(restored.send_to_box_half, 0);
    }
}
