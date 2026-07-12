/// 1:1 translation of com.fumbbl.ffb.server.net.CommandSocket (Jetty → axum WebSocket).
///
/// Java uses Jetty `@WebSocket` annotations with `onOpen`, `onClose`, `onMessage`.
/// Rust uses axum's `WebSocketUpgrade` extractor + per-connection async tasks.
///
/// `handle_connection`'s disconnect cleanup enqueues `AnyInternalServerCommand::SocketClosed`
/// onto `AppState::dispatch_tx` — the same queue `ServerCommunication`'s dispatch loop already
/// drains into `ServerCommandHandlerFactory::handle_command` — rather than calling
/// `SessionManager::remove_session` directly, matching Java's `CommandSocket.onClose()` →
/// `ServerCommunication.close(session)` → enqueued `InternalServerCommandSocketClosed` flow
/// (see `ServerCommunication::close`'s own doc comment for why `pSession.close()` itself is
/// modeled separately, by simply letting this task's outgoing sender drop).
use std::sync::{Arc, Mutex};
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
use tokio::sync::mpsc;
use ffb_model::model::ClientMode;
use ffb_protocol::client_commands::ClientCommand;
use crate::game_cache::GameCache;
use crate::model::received_command::{ReceivedCommand, SessionId};
use crate::net::commands::any_internal_server_command::AnyInternalServerCommand;
use crate::net::commands::internal_server_command_socket_closed::InternalServerCommandSocketClosed;
use crate::net::session_manager::SessionManager;

static NEXT_SESSION_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

fn next_session_id() -> SessionId {
    NEXT_SESSION_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

/// Shared server state passed to each axum handler via `State`.
#[derive(Clone)]
pub struct AppState {
    pub game_cache: Arc<Mutex<GameCache>>,
    pub session_manager: Arc<Mutex<SessionManager>>,
    pub dispatch_tx: mpsc::UnboundedSender<ReceivedCommand>,
}

/// axum route handler: `GET /`
///
/// Java: `CommandSocket` — the `@WebSocket`-annotated endpoint class.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_connection(socket, state))
}

/// Per-connection handler — runs for the lifetime of one WebSocket connection.
///
/// Java: `CommandSocket.onOpen()` / `onMessage()` / `onClose()` lifecycle.
async fn handle_connection(mut socket: WebSocket, state: AppState) {
    let session_id = next_session_id();
    log::debug!("WebSocket session {} opened", session_id);

    // Send server version handshake (Java: ServerCommandVersion)
    let version_msg = serde_json::json!({
        "netCommandId": "serverVersion",
        "version": env!("CARGO_PKG_VERSION"),
        "versionType": "release"
    });
    if socket.send(Message::Text(version_msg.to_string().into())).await.is_err() {
        return;
    }

    // Wait for the first ClientJoin command
    let join_data = loop {
        match socket.recv().await {
            Some(Ok(Message::Text(text))) => {
                match serde_json::from_str::<ClientCommand>(&text) {
                    Ok(ClientCommand::ClientJoin(join)) => break join,
                    Ok(other) => {
                        log::warn!("session {} sent {:?} before ClientJoin — ignoring", session_id, other);
                    }
                    Err(e) => {
                        log::warn!("session {} bad JSON before join: {}", session_id, e);
                    }
                }
            }
            Some(Ok(Message::Close(_))) | None => {
                log::debug!("session {} closed before joining", session_id);
                return;
            }
            _ => {}
        }
    };

    // Determine game ID (create slot on first join)
    let game_id = {
        let game_name = &join_data.game_id;
        let mut gc = state.game_cache.lock().unwrap();
        if let Some(gs) = gc.get_game_state_by_name(game_name) {
            gs.get_id()
        } else {
            let id = gc.create_game_state();
            gc.map_game_name_to_id(game_name.clone(), id);
            id
        }
    };

    // First coach to join a game becomes home; second becomes away.
    let home_coach = {
        let sm = state.session_manager.lock().unwrap();
        sm.get_session_of_home_coach(game_id).is_none()
    };

    // Outgoing channel for this session (replaces Jetty Session.getRemote())
    let (out_tx, mut out_rx) = mpsc::unbounded_channel::<String>();

    {
        let mut sm = state.session_manager.lock().unwrap();
        sm.add_session(
            session_id,
            game_id,
            join_data.coach.clone(),
            ClientMode::PLAYER,
            home_coach,
            vec![],
            out_tx,
        );
    }

    log::info!(
        "session {} joined game {} as coach {} ({})",
        session_id, game_id, join_data.coach,
        if home_coach { "home" } else { "away" }
    );

    // Main event loop: mux between incoming WS messages and outgoing sends.
    loop {
        tokio::select! {
            // Incoming: parse ClientCommand and enqueue for dispatch
            incoming = socket.recv() => {
                match incoming {
                    Some(Ok(Message::Text(text))) => {
                        match serde_json::from_str::<ClientCommand>(&text) {
                            Ok(cmd) => {
                                let _ = state.dispatch_tx.send(ReceivedCommand::new(cmd, session_id));
                            }
                            Err(e) => {
                                log::warn!("session {} bad JSON: {} — msg: {}", session_id, e, text);
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Err(e)) => {
                        log::warn!("session {} recv error: {}", session_id, e);
                        break;
                    }
                    _ => {}
                }
            }
            // Outgoing: flush messages queued by ServerCommunication
            out = out_rx.recv() => {
                match out {
                    Some(text) => {
                        if socket.send(Message::Text(text.into())).await.is_err() {
                            break;
                        }
                    }
                    None => break,
                }
            }
        }
    }

    // Cleanup on disconnect.
    //
    // Java: `CommandSocket.onClose()` calls `getServer().getCommunication().close(session)`,
    // which itself enqueues `new ReceivedCommand(new InternalServerCommandSocketClosed(),
    // session)` onto the same dispatch queue `onMessage()` uses (see
    // `ServerCommunication::close`'s own doc comment) — it does not call
    // `SessionManager.removeSession` directly itself; that removal (plus the sketch-cleanup/
    // leave-broadcast/replay-control-handoff side effects) all live inside
    // `ServerCommandHandlerSocketClosed::handle_command`, which is what actually runs when
    // the enqueued command is dispatched. A bare `sm.remove_session(session_id)` here would
    // both duplicate that removal and skip those other side effects entirely, so this now
    // enqueues the same internal command Java's `close()` does instead of removing the
    // session directly.
    let _ = state.dispatch_tx.send(ReceivedCommand::new_internal(
        AnyInternalServerCommand::SocketClosed(InternalServerCommandSocketClosed),
        session_id,
    ));
    log::debug!("WebSocket session {} closed", session_id);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_session_id_increases() {
        let a = next_session_id();
        let b = next_session_id();
        assert!(b > a);
    }

    #[test]
    fn app_state_is_clone() {
        let gc = Arc::new(Mutex::new(GameCache::new()));
        let sm = Arc::new(Mutex::new(SessionManager::new()));
        let (tx, _rx) = mpsc::unbounded_channel();
        let state = AppState { game_cache: gc, session_manager: sm, dispatch_tx: tx };
        let _clone = state.clone();
    }

    /// Proves `handle_connection`'s disconnect-cleanup statement — enqueuing
    /// `AnyInternalServerCommand::SocketClosed` on `AppState::dispatch_tx` — really reaches
    /// the real `ServerCommandHandlerSocketClosed` (via `ServerCommunication`'s live dispatch
    /// loop) rather than a bare `SessionManager::remove_session`, by running that exact
    /// statement against a session and observing the handler's own additional side effect
    /// (a `serverLeave` broadcast to a second, still-connected session in the same game —
    /// something a raw `remove_session` call could never produce on its own).
    #[tokio::test]
    async fn disconnect_cleanup_reaches_real_socket_closed_handler() {
        use crate::net::server_communication::ServerCommunication;
        use crate::db::db_connection_manager::DbConnectionManager;

        let gc = Arc::new(Mutex::new(GameCache::new()));
        let sm = Arc::new(Mutex::new(SessionManager::new()));
        let db = Arc::new(Mutex::new(DbConnectionManager::new()));
        let game_id = gc.lock().unwrap().create_game_state();

        let (tx1, _rx1) = mpsc::unbounded_channel();
        let (tx2, mut rx2) = mpsc::unbounded_channel();
        {
            let mut guard = sm.lock().unwrap();
            guard.add_session(1, game_id, "Home".into(), ClientMode::PLAYER, true, vec![], tx1);
            guard.add_session(2, game_id, "Away".into(), ClientMode::PLAYER, false, vec![], tx2);
        }

        let comms = ServerCommunication::new(Arc::clone(&gc), Arc::clone(&sm), db);
        let state = AppState {
            game_cache: Arc::clone(&gc),
            session_manager: Arc::clone(&sm),
            dispatch_tx: comms.sender(),
        };

        // This is the exact statement `handle_connection`'s cleanup section runs.
        let _ = state.dispatch_tx.send(ReceivedCommand::new_internal(
            AnyInternalServerCommand::SocketClosed(InternalServerCommandSocketClosed),
            1,
        ));

        for _ in 0..10 {
            tokio::task::yield_now().await;
        }

        assert_eq!(sm.lock().unwrap().get_game_id_for_session(1), 0, "session should have been removed");
        let msg = rx2.try_recv().expect("expected a real serverLeave broadcast from the handler");
        assert!(msg.contains("serverLeave"));
    }
}
