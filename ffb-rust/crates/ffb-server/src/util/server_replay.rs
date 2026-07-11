/// 1:1 translation of com.fumbbl.ffb.server.util.UtilServerReplay.
///
/// Java:
/// ```java
/// public static void startServerReplay(GameState pGameState, int pReplayToCommandNr, Session pSession) {
///     if ((pGameState == null) || (pSession == null)) {
///         return;
///     }
///     FantasyFootballServer server = pGameState.getServer();
///     if (server.getSessionManager().getGameIdForSession(pSession) != pGameState.getId()) {
///         server.getCommunication().sendGameState(pSession, pGameState);
///     }
///     server.getReplayer().add(new ServerReplay(pGameState, pReplayToCommandNr, pSession));
/// }
/// ```
///
/// Java reaches `SessionManager`/`ServerCommunication`/`ServerReplayer` via
/// `gameState.getServer().getX()`; per this crate's convention (see `game_cache.rs`,
/// `server_command_handler_password_challenge.rs`) they are threaded through explicitly
/// instead as parameters.
///
/// Phase AAB wires the previously-missing `ServerReplayer`/`ServerReplay` (from
/// `ffb-engine`) in for real: `game_state` now carries `(game_id, game_state_message,
/// game_log)` — the `GameLog` is what `ServerReplay::new` needs to snapshot/renumber the
/// replay's commands (see that struct's own doc comment for why a `GameLog` reference is
/// threaded through directly rather than a whole `GameState`).
use std::sync::Mutex;

use ffb_engine::game_log::GameLog;
use ffb_engine::server_replay::ServerReplay;
use ffb_engine::server_replayer::ServerReplayer;

use crate::model::received_command::SessionId;
use crate::net::session_manager::SessionManager;

/// Java: `startServerReplay(GameState, int, Session)`.
///
/// `game_state_id`/`game_state_message` stand in for the Java `GameState` (its `getId()`
/// and an already-serialized game-state message respectively — there is no
/// `ServerCommunication.sendGameState` in this crate, so the caller supplies the message
/// to send, matching how `ServerCommandHandlerPasswordChallenge` builds its own command
/// JSON before handing it to `SessionManager::send_to`).
pub fn start_server_replay(
    game_state: Option<(i64, &str, &GameLog)>,
    replay_to_command_nr: i32,
    session_id: Option<SessionId>,
    session_manager: &SessionManager,
    replayer: &Mutex<ServerReplayer>,
) {
    let (Some((game_id, game_state_message, game_log)), Some(session_id)) = (game_state, session_id) else {
        return;
    };

    if session_manager.get_game_id_for_session(session_id) != game_id {
        session_manager.send_to(session_id, game_state_message);
    }

    let replay = ServerReplay::new(game_id, replay_to_command_nr, session_id, game_log);
    replayer.lock().unwrap().add(replay);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::ClientMode;
    use tokio::sync::mpsc;

    fn setup_session_manager() -> (SessionManager, mpsc::UnboundedReceiver<String>) {
        let mut sm = SessionManager::new();
        let (tx, rx) = mpsc::unbounded_channel();
        sm.add_session(1, 100, "Coach".into(), ClientMode::PLAYER, true, vec![], tx);
        (sm, rx)
    }

    #[test]
    fn null_game_state_is_a_no_op() {
        let (sm, mut rx) = setup_session_manager();
        let replayer = Mutex::new(ServerReplayer::new());
        start_server_replay(None, 5, Some(1), &sm, &replayer);
        assert!(rx.try_recv().is_err());
        assert_eq!(replayer.lock().unwrap().queue_size(), 0);
    }

    #[test]
    fn null_session_is_a_no_op() {
        let (sm, mut rx) = setup_session_manager();
        let log = GameLog::new();
        let replayer = Mutex::new(ServerReplayer::new());
        start_server_replay(Some((100, "game-state-json", &log)), 5, None, &sm, &replayer);
        assert!(rx.try_recv().is_err());
        assert_eq!(replayer.lock().unwrap().queue_size(), 0);
    }

    #[test]
    fn sends_game_state_when_session_is_tracking_a_different_game() {
        let (sm, mut rx) = setup_session_manager();
        let log = GameLog::new();
        let replayer = Mutex::new(ServerReplayer::new());
        // Session 1 is tracking game 100 (see setup); simulate replay into a different game.
        start_server_replay(Some((999, "game-state-json", &log)), 5, Some(1), &sm, &replayer);
        let msg = rx.try_recv().expect("expected sendGameState to fire");
        assert_eq!(msg, "game-state-json");
    }

    #[test]
    fn does_not_resend_game_state_when_already_tracking_the_same_game() {
        let (sm, mut rx) = setup_session_manager();
        let log = GameLog::new();
        let replayer = Mutex::new(ServerReplayer::new());
        start_server_replay(Some((100, "game-state-json", &log)), 5, Some(1), &sm, &replayer);
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn adds_a_replay_to_the_replayer_queue() {
        let (sm, _rx) = setup_session_manager();
        let log = GameLog::new();
        let replayer = Mutex::new(ServerReplayer::new());
        start_server_replay(Some((100, "game-state-json", &log)), 5, Some(1), &sm, &replayer);
        assert_eq!(replayer.lock().unwrap().queue_size(), 1);
    }
}
