/// 1:1 translation of com.fumbbl.ffb.server.handler.ServerCommandHandlerReplayLoaded.
use std::sync::{Arc, Mutex};
use ffb_model::enums::{NetCommandId, ServerStatus};
use ffb_protocol::commands::server_command_status::ServerCommandStatus;
use crate::game_cache::GameCache;
use crate::model::received_command::SessionId;
use crate::net::commands::internal_server_command::InternalServerCommand;
use crate::net::commands::internal_server_command_replay_loaded::InternalServerCommandReplayLoaded;
use crate::net::session_manager::SessionManager;

pub struct ServerCommandHandlerReplayLoaded {
    game_cache: Arc<Mutex<GameCache>>,
    session_manager: Arc<Mutex<SessionManager>>,
}

impl ServerCommandHandlerReplayLoaded {
    pub fn new(game_cache: Arc<Mutex<GameCache>>, session_manager: Arc<Mutex<SessionManager>>) -> Self {
        Self { game_cache, session_manager }
    }

    /// Java: getId() — returns NetCommandId for REPLAY_LOADED.
    pub fn get_id(&self) -> NetCommandId {
        NetCommandId::InternalServerReplayLoaded
    }

    /// Java: `handleCommand(ReceivedCommand)` — handles notification that a replay has loaded.
    ///
    /// If the game is found in the cache, Java marks it `GameStatus.REPLAYING`
    /// and starts the replay via `UtilServerReplay.startServerReplay`; neither
    /// a status field on the server-side `GameState` wrapper nor the replay
    /// engine exist in the Rust MVP yet, so that branch stays a narrow todo.
    /// The "game not found" branch (send an `ERROR_UNKNOWN_GAME_ID` status to
    /// the session) is fully real.
    pub fn handle_command(&self, cmd: &InternalServerCommandReplayLoaded, session_id: SessionId) -> bool {
        if cmd.get_game_id() > 0 {
            let found = {
                let gc = self.game_cache.lock().unwrap();
                gc.get_game_state_by_id(cmd.get_game_id()).is_some()
            };

            if found {
                // Java: gameState.setStatus(GameStatus.REPLAYING);
                //       UtilServerReplay.startServerReplay(gameState, replayToCommandNr, session);
                todo!("Phase ZV: GameState.setStatus(REPLAYING) + UtilServerReplay.startServerReplay need wiring")
            } else {
                let status = ServerCommandStatus::new(
                    ServerStatus::ErrorUnknownGameId,
                    ServerStatus::ErrorUnknownGameId.message(),
                );
                let json = status.to_json_value().to_string();
                let sm = self.session_manager.lock().unwrap();
                sm.send_to(session_id, &json);
            }
        }

        true
    }
}

impl Default for ServerCommandHandlerReplayLoaded {
    fn default() -> Self {
        Self::new(Arc::new(Mutex::new(GameCache::new())), Arc::new(Mutex::new(SessionManager::new())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::ClientMode;
    use tokio::sync::mpsc;

    #[test]
    fn construct() {
        let _ = ServerCommandHandlerReplayLoaded::default();
    }

    #[test]
    fn get_id_returns_internal_server_replay_loaded() {
        let h = ServerCommandHandlerReplayLoaded::default();
        assert_eq!(h.get_id(), NetCommandId::InternalServerReplayLoaded);
    }

    #[test]
    fn handle_command_with_zero_game_id_is_noop() {
        let h = ServerCommandHandlerReplayLoaded::default();
        let cmd = InternalServerCommandReplayLoaded::new(0, 0, "coach".to_string());
        assert!(h.handle_command(&cmd, 1));
    }

    #[test]
    fn handle_command_unknown_game_sends_error_status() {
        let gc = Arc::new(Mutex::new(GameCache::new()));
        let sm_arc = Arc::new(Mutex::new(SessionManager::new()));
        let (tx, mut rx) = mpsc::unbounded_channel();
        {
            let mut sm = sm_arc.lock().unwrap();
            sm.add_session(7, 0, "Coach".into(), ClientMode::PLAYER, true, vec![], tx);
        }
        let h = ServerCommandHandlerReplayLoaded::new(gc, sm_arc);
        let cmd = InternalServerCommandReplayLoaded::new(999, 0, "coach".to_string());
        assert!(h.handle_command(&cmd, 7));
        let sent = rx.try_recv().expect("expected an ERROR_UNKNOWN_GAME_ID status message");
        assert!(sent.contains("Unknown Game Id"));
    }

    #[test]
    fn handle_command_known_game_hits_replay_engine_todo() {
        let gc = Arc::new(Mutex::new(GameCache::new()));
        let game_id = { gc.lock().unwrap().create_game_state() };
        let sm = Arc::new(Mutex::new(SessionManager::new()));
        let h = ServerCommandHandlerReplayLoaded::new(gc, sm);
        let cmd = InternalServerCommandReplayLoaded::new(game_id, 5, "coach".to_string());
        let result = std::panic::catch_unwind(|| h.handle_command(&cmd, 1));
        assert!(result.is_err(), "found branch requires GameState.status + replay engine wiring (narrow todo!)");
    }
}
