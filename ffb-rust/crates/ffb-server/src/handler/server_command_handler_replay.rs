/// 1:1 translation of com.fumbbl.ffb.server.handler.ServerCommandHandlerReplay.
use std::sync::{Arc, Mutex};
use ffb_model::enums::NetCommandId;
use ffb_protocol::commands::client_command_replay::ClientCommandReplay;
use crate::game_cache::GameCache;
use crate::model::received_command::SessionId;
use crate::net::session_manager::SessionManager;

pub struct ServerCommandHandlerReplay {
    game_cache: Arc<Mutex<GameCache>>,
    session_manager: Arc<Mutex<SessionManager>>,
}

impl ServerCommandHandlerReplay {
    pub fn new(game_cache: Arc<Mutex<GameCache>>, session_manager: Arc<Mutex<SessionManager>>) -> Self {
        Self { game_cache, session_manager }
    }

    /// Java: getId() — returns NetCommandId for REPLAY.
    pub fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientReplay
    }

    /// Java: `handleCommand(ReceivedCommand)` — handles a replay request.
    ///
    /// Resolves the target game id (from the command, or from the session if
    /// the command carries none) and looks it up in the `GameCache`. The
    /// actual replay-playback engine (Java: `UtilServerReplay.startServerReplay`,
    /// which needs a live `Replayer` + `Communication.sendGameState`) and the
    /// DB re-query / backup-service fallback (Java: `GameCache.queryFromDb` +
    /// `ServerRequestLoadReplay` enqueued on the `ServerRequestProcessor`) are
    /// not wired in the Rust MVP yet.
    pub fn handle_command(&self, cmd: &ClientCommandReplay, session_id: SessionId) -> bool {
        let mut game_id = cmd.get_game_id();

        if game_id == 0 {
            let sm = self.session_manager.lock().unwrap();
            game_id = sm.get_game_id_for_session(session_id);
        }

        if game_id == 0 {
            return false;
        }

        let found = {
            let gc = self.game_cache.lock().unwrap();
            gc.get_game_state_by_id(game_id).is_some()
        };

        if found {
            // Java: UtilServerReplay.startServerReplay(gameState, replayToCommandNr, session)
            todo!("Phase ZV: UtilServerReplay.startServerReplay needs Replayer + Communication.sendGameState wiring")
        } else {
            // Java: GameCache.queryFromDb(gameId) then ServerRequestLoadReplay enqueued on
            // the ServerRequestProcessor if still not found — both need DB/queue wiring.
            todo!("Phase ZV: GameCache.queryFromDb + ServerRequestLoadReplay need DB/queue wiring")
        }
    }
}

impl Default for ServerCommandHandlerReplay {
    fn default() -> Self {
        Self::new(Arc::new(Mutex::new(GameCache::new())), Arc::new(Mutex::new(SessionManager::new())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let _ = ServerCommandHandlerReplay::default();
    }

    #[test]
    fn get_id_returns_client_replay() {
        let h = ServerCommandHandlerReplay::default();
        assert_eq!(h.get_id(), NetCommandId::ClientReplay);
    }

    #[test]
    fn handle_command_returns_false_when_no_game_id_available() {
        let h = ServerCommandHandlerReplay::default();
        // Command has gameId == 0 and the session (99) was never registered,
        // so getGameIdForSession also returns 0 — Java returns false here.
        let cmd = ClientCommandReplay::new();
        assert!(!h.handle_command(&cmd, 99));
    }

    #[test]
    fn handle_command_resolves_game_id_from_session_when_command_has_none() {
        use ffb_model::model::ClientMode;
        use tokio::sync::mpsc;

        let gc = Arc::new(Mutex::new(GameCache::new()));
        let sm_arc = Arc::new(Mutex::new(SessionManager::new()));
        {
            let (tx, _rx) = mpsc::unbounded_channel();
            let mut sm = sm_arc.lock().unwrap();
            sm.add_session(1, 42, "Coach".into(), ClientMode::PLAYER, true, vec![], tx);
        }
        let h = ServerCommandHandlerReplay::new(gc, sm_arc);
        let cmd = ClientCommandReplay::new();
        // Game 42 is not in the cache, so the not-found (DB/queue) branch is hit.
        let result = std::panic::catch_unwind(|| h.handle_command(&cmd, 1));
        assert!(result.is_err(), "not-found branch requires DB/queue wiring (narrow todo!)");
    }

    #[test]
    fn handle_command_found_game_hits_replay_engine_todo() {
        let gc = Arc::new(Mutex::new(GameCache::new()));
        let game_id = { gc.lock().unwrap().create_game_state() };
        let sm = Arc::new(Mutex::new(SessionManager::new()));
        let h = ServerCommandHandlerReplay::new(gc, sm);
        let cmd = ClientCommandReplay::with_params(game_id, 0, "coach");
        let result = std::panic::catch_unwind(|| h.handle_command(&cmd, 1));
        assert!(result.is_err(), "found branch requires Replayer wiring (narrow todo!)");
    }
}
