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
/// `ServerReplayer`/`ServerReplay` (the `server.getReplayer().add(...)` step) have no
/// Rust equivalent in this crate — there is no ported replay-playback engine (confirmed
/// by grep: no `ServerReplay`/`Replayer` struct exists under `crates/ffb-server/src`, only
/// the unrelated `ReplaySessionManager` which tracks *sessions* watching a shared replay,
/// not command-log playback). Building that subsystem is out of scope here (same
/// documented gap as `ServerCommandHandlerJoinReplay`'s `ReplayCache`/`ReplayState` tail),
/// so `start_server_replay` performs the real, portable half of the method — the null
/// checks and the "send game state if not already tracking this game" call — and leaves
/// the replayer registration as a documented no-op.
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
    game_state: Option<(i64, &str)>,
    replay_to_command_nr: i32,
    session_id: Option<SessionId>,
    session_manager: &SessionManager,
) {
    let (game_id, game_state_message) = match (game_state, session_id) {
        (Some(gs), Some(_)) => gs,
        _ => return,
    };
    let session_id = session_id.unwrap();

    if session_manager.get_game_id_for_session(session_id) != game_id {
        session_manager.send_to(session_id, game_state_message);
    }

    // Java: `server.getReplayer().add(new ServerReplay(pGameState, pReplayToCommandNr, pSession))`.
    // No `ServerReplayer`/`ServerReplay` exists in this crate — see module doc comment.
    let _ = replay_to_command_nr;
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
        start_server_replay(None, 5, Some(1), &sm);
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn null_session_is_a_no_op() {
        let (sm, mut rx) = setup_session_manager();
        start_server_replay(Some((100, "game-state-json")), 5, None, &sm);
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn sends_game_state_when_session_is_tracking_a_different_game() {
        let (sm, mut rx) = setup_session_manager();
        // Session 1 is tracking game 100 (see setup); simulate replay into a different game.
        start_server_replay(Some((999, "game-state-json")), 5, Some(1), &sm);
        let msg = rx.try_recv().expect("expected sendGameState to fire");
        assert_eq!(msg, "game-state-json");
    }

    #[test]
    fn does_not_resend_game_state_when_already_tracking_the_same_game() {
        let (sm, mut rx) = setup_session_manager();
        start_server_replay(Some((100, "game-state-json")), 5, Some(1), &sm);
        assert!(rx.try_recv().is_err());
    }
}
