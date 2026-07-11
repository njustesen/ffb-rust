/// 1:1 translation of com.fumbbl.ffb.server.net.CommandSocket (Jetty → axum WebSocket).
///
/// Java uses Jetty `@WebSocket` annotations with `onOpen`, `onClose`, `onMessage`.
/// Rust uses axum's `WebSocketUpgrade` extractor + per-connection async tasks.
use std::sync::{Arc, Mutex};
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
use tokio::sync::mpsc;
use ffb_model::model::ClientMode;
use ffb_protocol::client_commands::ClientCommand;
use crate::game_cache::GameCache;
use crate::model::received_command::{ReceivedCommand, SessionId};
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

    // Cleanup on disconnect
    {
        let mut sm = state.session_manager.lock().unwrap();
        sm.remove_session(session_id);
    }
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
}
