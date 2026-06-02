use serde::{Deserialize, Serialize};
use ffb_model::model::game::Game;
use ffb_model::enums::{GameStatus, NetCommandId};

/// Commands sent from the Java server to the Rust client.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "netCommandId", rename_all = "camelCase")]
pub enum ServerCommand {
    /// Full game state snapshot (sent on join, after timeout, etc.).
    ServerGameState(ServerGameState),
    /// Incremental model change list (sent after each step).
    ServerModelSync(ServerModelSync),
    /// Current game time / turn progress.
    ServerGameTime(ServerGameTime),
    /// Game status update.
    ServerStatus(ServerStatus),
    /// A coach joined.
    ServerJoin(ServerJoin),
    /// A coach left.
    ServerLeave(ServerLeave),
    /// Chat message.
    ServerTalk(ServerTalk),
    /// Admin message.
    ServerAdminMessage(ServerAdminMessage),
    /// Sound effect cue.
    ServerSound(ServerSound),
    /// Keep-alive pong.
    ServerPong(ServerPong),
    /// Password challenge.
    ServerPasswordChallenge(ServerPasswordChallenge),
    /// Server version info.
    ServerVersion(ServerVersion),
    /// A player was added (e.g. journeyman).
    ServerAddPlayer(ServerAddPlayer),
    /// A player was zapped.
    ServerZapPlayer(ServerZapPlayer),
    /// A player was un-zapped.
    ServerUnzapPlayer(ServerUnzapPlayer),
    /// Game list (lobby).
    ServerGameList(ServerGameList),
    /// Team list.
    ServerTeamList(ServerTeamList),
}

// ── Individual server command structs ─────────────────────────────────────────

/// Full game state snapshot — the most critical server command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerGameState {
    pub command_nr: i64,
    pub game: Box<Game>,
}

/// Incremental model changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerModelSync {
    pub command_nr: i64,
    pub changes: Vec<ModelChange>,
}

/// A single model change record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelChange {
    pub id: String,
    pub data_type: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerGameTime {
    pub half: i32,
    pub turn_nr: i32,
    pub seconds_left: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStatus {
    pub status: GameStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerJoin {
    pub coach: String,
    pub team_id: String,
    pub side: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerLeave {
    pub coach: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerTalk {
    pub coach: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerAdminMessage {
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSound {
    pub sound_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerPong {
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerPasswordChallenge {
    pub challenge: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerVersion {
    pub server_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerAddPlayer {
    pub player: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerZapPlayer {
    pub player_id: String,
    pub position_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerUnzapPlayer {
    pub player_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerGameList {
    pub games: Vec<GameListEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameListEntry {
    pub game_id: String,
    pub home_team: String,
    pub away_team: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerTeamList {
    pub teams: Vec<TeamListEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamListEntry {
    pub team_id: String,
    pub name: String,
    pub coach: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rt(cmd: &ServerCommand) {
        let json = serde_json::to_string(cmd).unwrap();
        let _back: ServerCommand = serde_json::from_str(&json)
            .unwrap_or_else(|e| panic!("round-trip failed: {e}\njson={json}"));
    }

    #[test]
    fn server_game_time_round_trip() {
        rt(&ServerCommand::ServerGameTime(ServerGameTime {
            half: 1, turn_nr: 3, seconds_left: 90,
        }));
    }

    #[test]
    fn server_status_round_trip() {
        rt(&ServerCommand::ServerStatus(ServerStatus {
            status: GameStatus::Active,
        }));
    }

    #[test]
    fn server_talk_round_trip() {
        rt(&ServerCommand::ServerTalk(ServerTalk {
            coach: "Coach1".into(),
            message: "Hello!".into(),
        }));
    }

    #[test]
    fn server_join_round_trip() {
        rt(&ServerCommand::ServerJoin(ServerJoin {
            coach: "TestCoach".into(), team_id: "team1".into(), side: "home".into(),
        }));
    }

    #[test]
    fn server_pong_round_trip() {
        rt(&ServerCommand::ServerPong(ServerPong { timestamp: 1234567890 }));
    }

    #[test]
    fn server_game_list_round_trip() {
        rt(&ServerCommand::ServerGameList(ServerGameList {
            games: vec![GameListEntry {
                game_id: "g1".into(),
                home_team: "home".into(),
                away_team: "away".into(),
                status: "Active".into(),
            }],
        }));
    }

    #[test]
    fn server_tag_is_camel_case() {
        let json = serde_json::to_string(&ServerCommand::ServerPong(ServerPong { timestamp: 0 })).unwrap();
        assert!(json.contains("serverPong"), "tag must be camelCase, got: {json}");
    }
}
