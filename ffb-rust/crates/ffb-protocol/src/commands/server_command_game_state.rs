use serde::{Deserialize, Serialize};
use ffb_model::model::game::Game;
use ffb_model::enums::NetCommandId;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandGameState`.
/// Carries a full `Game` snapshot; sent on join and after reconnects.
/// Java: extends `ServerCommand`; `isReplayable()` returns `false`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCommandGameState {
    /// Java: `fCommandNr` inherited from `ServerCommand`.
    pub command_nr: i32,
    /// Java: `fGame` — the complete game state.
    pub game: Option<Box<Game>>,
}

impl ServerCommandGameState {
    pub const ID: NetCommandId = NetCommandId::ServerGameState;

    pub fn new(game: Option<Game>) -> Self {
        Self {
            command_nr: 0,
            game: game.map(Box::new),
        }
    }

    /// Java: `isReplayable()` — game state snapshots are NOT stored in replays.
    pub fn is_replayable(&self) -> bool {
        false
    }

    pub fn id(&self) -> NetCommandId {
        Self::ID
    }
}

impl Default for ServerCommandGameState {
    fn default() -> Self {
        Self::new(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_replayable() {
        assert!(!ServerCommandGameState::default().is_replayable());
    }

    #[test]
    fn id_is_server_game_state() {
        assert_eq!(ServerCommandGameState::default().id(), NetCommandId::ServerGameState);
    }

    #[test]
    fn new_with_no_game() {
        let cmd = ServerCommandGameState::new(None);
        assert!(cmd.game.is_none());
    }

    #[test]
    fn serde_round_trip_no_game() {
        let cmd = ServerCommandGameState { command_nr: 5, game: None };
        let json = serde_json::to_string(&cmd).unwrap();
        let back: ServerCommandGameState = serde_json::from_str(&json).unwrap();
        assert_eq!(back.command_nr, 5);
        assert!(back.game.is_none());
    }
}
