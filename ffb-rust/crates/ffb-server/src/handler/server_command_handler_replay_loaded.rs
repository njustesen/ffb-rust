/// 1:1 translation of com.fumbbl.ffb.server.handler.ServerCommandHandlerReplayLoaded.
use std::sync::{Arc, Mutex};
use ffb_engine::server_replayer::ServerReplayer;
use ffb_model::enums::{GameStatus, NetCommandId, ServerStatus};
use ffb_protocol::commands::server_command_game_state::ServerCommandGameState;
use ffb_protocol::commands::server_command_status::ServerCommandStatus;
use crate::game_cache::GameCache;
use crate::model::received_command::SessionId;
use crate::net::commands::internal_server_command::InternalServerCommand;
use crate::net::commands::internal_server_command_replay_loaded::InternalServerCommandReplayLoaded;
use crate::net::session_manager::SessionManager;
use crate::util::server_replay::start_server_replay;

pub struct ServerCommandHandlerReplayLoaded {
    game_cache: Arc<Mutex<GameCache>>,
    session_manager: Arc<Mutex<SessionManager>>,
    replayer: Arc<Mutex<ServerReplayer>>,
}

impl ServerCommandHandlerReplayLoaded {
    pub fn new(
        game_cache: Arc<Mutex<GameCache>>,
        session_manager: Arc<Mutex<SessionManager>>,
        replayer: Arc<Mutex<ServerReplayer>>,
    ) -> Self {
        Self { game_cache, session_manager, replayer }
    }

    /// Java: getId() — returns NetCommandId for REPLAY_LOADED.
    pub fn get_id(&self) -> NetCommandId {
        NetCommandId::InternalServerReplayLoaded
    }

    /// Java: `handleCommand(ReceivedCommand)` — handles notification that a replay has loaded.
    ///
    /// ```java
    /// if (replayCommand.getGameId() > 0) {
    ///     GameState gameState = getServer().getGameCache().getGameStateById(replayCommand.getGameId());
    ///     if (gameState != null) {
    ///         gameState.setStatus(GameStatus.REPLAYING);
    ///         UtilServerReplay.startServerReplay(gameState, replayCommand.getReplayToCommandNr(), pReceivedCommand.getSession());
    ///     } else {
    ///         getServer().getCommunication().sendStatus(pReceivedCommand.getSession(), ServerStatus.ERROR_UNKNOWN_GAME_ID, null);
    ///     }
    /// }
    /// return true;
    /// ```
    ///
    /// Both branches are fully real: the found branch marks the game
    /// `GameStatus::Replaying` and starts the replay for real via
    /// `start_server_replay` (Phase AAB); the not-found branch sends
    /// `ERROR_UNKNOWN_GAME_ID` as before.
    pub fn handle_command(&self, cmd: &InternalServerCommandReplayLoaded, session_id: SessionId) -> bool {
        if cmd.get_game_id() > 0 {
            let mut gc = self.game_cache.lock().unwrap();
            match gc.get_game_state_by_id_mut(cmd.get_game_id()) {
                Some(game_state) => {
                    game_state.set_status(GameStatus::Replaying);
                    let message = ServerCommandGameState::new(game_state.get_game().cloned())
                        .to_json_value()
                        .to_string();
                    let sm = self.session_manager.lock().unwrap();
                    start_server_replay(
                        Some((game_state.get_id(), message.as_str(), &game_state.game_log)),
                        cmd.get_replay_to_command_nr(),
                        Some(session_id),
                        &sm,
                        &self.replayer,
                    );
                }
                None => {
                    drop(gc);
                    let status = ServerCommandStatus::new(
                        ServerStatus::ErrorUnknownGameId,
                        ServerStatus::ErrorUnknownGameId.message(),
                    );
                    let json = status.to_json_value().to_string();
                    let sm = self.session_manager.lock().unwrap();
                    sm.send_to(session_id, &json);
                }
            }
        }

        true
    }
}

impl Default for ServerCommandHandlerReplayLoaded {
    fn default() -> Self {
        Self::new(
            Arc::new(Mutex::new(GameCache::new())),
            Arc::new(Mutex::new(SessionManager::new())),
            Arc::new(Mutex::new(ServerReplayer::new())),
        )
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
        let h = ServerCommandHandlerReplayLoaded::new(gc, sm_arc, Arc::new(Mutex::new(ServerReplayer::new())));
        let cmd = InternalServerCommandReplayLoaded::new(999, 0, "coach".to_string());
        assert!(h.handle_command(&cmd, 7));
        let sent = rx.try_recv().expect("expected an ERROR_UNKNOWN_GAME_ID status message");
        assert!(sent.contains("Unknown Game Id"));
    }

    #[test]
    fn handle_command_known_game_sets_status_and_starts_replay() {
        let gc = Arc::new(Mutex::new(GameCache::new()));
        let game_id = { gc.lock().unwrap().create_game_state() };
        let sm = Arc::new(Mutex::new(SessionManager::new()));
        let replayer = Arc::new(Mutex::new(ServerReplayer::new()));
        let h = ServerCommandHandlerReplayLoaded::new(Arc::clone(&gc), Arc::clone(&sm), Arc::clone(&replayer));
        let cmd = InternalServerCommandReplayLoaded::new(game_id, 5, "coach".to_string());
        assert!(h.handle_command(&cmd, 1));

        assert_eq!(
            gc.lock().unwrap().get_game_state_by_id(game_id).unwrap().get_status(),
            Some(GameStatus::Replaying)
        );
        assert_eq!(replayer.lock().unwrap().queue_size(), 1);
    }

    #[test]
    fn handle_command_known_game_sends_game_state_when_session_tracks_different_game() {
        let gc = Arc::new(Mutex::new(GameCache::new()));
        let game_id = { gc.lock().unwrap().create_game_state() };
        let sm = Arc::new(Mutex::new(SessionManager::new()));
        let (tx, mut rx) = mpsc::unbounded_channel();
        sm.lock().unwrap().add_session(1, 0, "Coach".into(), ClientMode::PLAYER, true, vec![], tx);
        let replayer = Arc::new(Mutex::new(ServerReplayer::new()));
        let h = ServerCommandHandlerReplayLoaded::new(Arc::clone(&gc), Arc::clone(&sm), replayer);
        let cmd = InternalServerCommandReplayLoaded::new(game_id, 0, "coach".to_string());
        assert!(h.handle_command(&cmd, 1));
        let msg = rx.try_recv().expect("expected a serverGameState message");
        assert!(msg.contains("serverGameState"));
    }
}
