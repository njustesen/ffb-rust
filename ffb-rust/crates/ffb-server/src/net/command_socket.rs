/// 1:1 translation of com.fumbbl.ffb.server.net.CommandSocket (Jetty → axum WebSocket).
///
/// Java's `CommandSocket` only *receives*: `onBinaryMessage` decodes the UTF-8 bytes and
/// forwards to `onTextMessage`, which LZString-decompresses (when command compression is on),
/// parses the JSON, builds the `NetCommand` via `NetCommandFactory.forJsonValue` and enqueues
/// a `ReceivedCommand`; `onClose` enqueues `InternalServerCommandSocketClosed`. Sending lives in
/// `ServerCommunication.send(Session, NetCommand)`, which serializes `command.toJsonValue()`,
/// LZString-compresses it when compression is on, and writes the UTF-8 bytes as one BINARY
/// frame. Jetty gives every connection a `Session` before any command arrives; here each
/// connection registers its outgoing channel with `SessionManager::register_socket` on
/// connect, so handlers can answer a session that has not joined a game yet (version
/// request, password challenge, team list) exactly as Java's `send(session, ...)` can. The
/// per-connection writer task below is the `session.getRemote().sendBytesByFuture(...)` half
/// of `ServerCommunication.send` — handlers hand it the `toJsonValue().toString()` text.
use std::sync::{Arc, Mutex};
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
use tokio::sync::mpsc;
use ffb_model::json::lz_string::{compress_to_utf16, decompress_from_utf16};
use ffb_protocol::net_command_factory::{AnyNetCommand, NetCommandFactory};
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
    /// Java: `CommandSocket.fCommandCompression` / `ServerCommunication.fCommandCompression`,
    /// both from `IServerProperty.SERVER_COMMAND_COMPRESSION`.
    pub command_compression: bool,
}

/// axum route handler for `/command` (Java: `CommandServlet` → `new CommandSocket(...)`).
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_connection(socket, state))
}

/// Java: `CommandSocket.onTextMessage(Session, String)` minus the enqueue — returns the
/// decoded client command, or `None` where Java returns early / logs and drops the message
/// (`netCommand == null`, malformed JSON, failed decompression).
pub fn decode_text_message(text: &str, command_compression: bool) -> Option<AnyNetCommand> {
    let decompressed = if command_compression {
        decompress_from_utf16(text)?
    } else {
        text.to_string()
    };
    let json_value: serde_json::Value = serde_json::from_str(&decompressed).ok()?;
    NetCommandFactory::new().for_json_value(&json_value)
}

/// Java: the payload half of `ServerCommunication.send(Session, NetCommand)` — the
/// `toJsonValue().toString()` text, LZString-compressed when compression is on, as the UTF-8
/// bytes of one BINARY frame (`sendBytesByFuture(ByteBuffer.wrap(text.getBytes(UTF_8)))`).
/// `None` where Java skips the send (`!StringTool.isProvided(textMessage)`).
pub fn encode_text_message(json_text: &str, command_compression: bool) -> Option<Vec<u8>> {
    let text_message = if command_compression {
        compress_to_utf16(json_text)
    } else {
        json_text.to_string()
    };
    if text_message.is_empty() {
        return None;
    }
    Some(text_message.into_bytes())
}

/// Per-connection lifecycle: Java `onConnect` / `onBinaryMessage` / `onTextMessage` / `onClose`.
async fn handle_connection(mut socket: WebSocket, state: AppState) {
    let session_id = next_session_id();
    log::debug!("WebSocket session {} opened", session_id);

    // The Jetty `Session` exists from connect on: register its outgoing channel.
    let (out_tx, mut out_rx) = mpsc::unbounded_channel::<String>();
    state.session_manager.lock().unwrap().register_socket(session_id, out_tx);

    loop {
        tokio::select! {
            incoming = socket.recv() => {
                let text = match incoming {
                    // Java: `onBinaryMessage` -> `new String(buf, offset, length, UTF_8)`.
                    Some(Ok(Message::Binary(bytes))) => String::from_utf8_lossy(&bytes).into_owned(),
                    Some(Ok(Message::Text(text))) => text.to_string(),
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Err(e)) => {
                        log::warn!("session {} recv error: {}", session_id, e);
                        break;
                    }
                    _ => continue,
                };
                match decode_text_message(&text, state.command_compression) {
                    Some(AnyNetCommand::Client(cmd)) => {
                        log::debug!("session {} <- {:?}", session_id, ffb_protocol::net_command::NetCommand::get_id(&cmd));
                        let _ = state.dispatch_tx.send(ReceivedCommand::new(cmd, session_id));
                    }
                    Some(AnyNetCommand::Server(_)) => {
                        log::warn!("session {} sent a server command — dropped", session_id);
                    }
                    None => log::warn!(
                        "session {} sent an undecodable message — dropped: {:?}",
                        session_id,
                        text.chars().take(80).collect::<String>()
                    ),
                }
            }
            out = out_rx.recv() => {
                match out {
                    Some(json_text) => {
                        log::debug!(
                            "session {} -> {}",
                            session_id,
                            json_text.chars().take(160).collect::<String>()
                        );
                        if let Some(bytes) = encode_text_message(&json_text, state.command_compression) {
                            if socket.send(Message::Binary(bytes.into())).await.is_err() {
                                break;
                            }
                        }
                    }
                    None => break,
                }
            }
        }
    }

    // Java: `onClose` -> `handleCommand(new ReceivedCommand(new InternalServerCommandSocketClosed(), session))`;
    // `ServerCommandHandlerSocketClosed` does the session removal and leave broadcast.
    let _ = state.dispatch_tx.send(ReceivedCommand::new_internal(
        AnyInternalServerCommand::SocketClosed(InternalServerCommandSocketClosed),
        session_id,
    ));
    log::debug!("WebSocket session {} closed", session_id);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::ClientMode;
    use ffb_protocol::commands::any_client_command::AnyClientCommand;

    #[test]
    fn next_session_id_increases() {
        let a = next_session_id();
        let b = next_session_id();
        assert!(b > a);
    }

    #[test]
    fn decodes_plain_java_command() {
        let cmd = decode_text_message(r#"{"netCommandId":"clientRequestVersion"}"#, false);
        assert!(matches!(cmd, Some(AnyNetCommand::Client(AnyClientCommand::ClientRequestVersion(_)))));
    }

    /// A compressed client frame (Java `CommandEndpoint.send` with compression on);
    /// `compress_to_utf16` is byte-identical to Java's (vectors in lz_string.rs).
    #[test]
    fn decodes_compressed_java_command() {
        let compressed = compress_to_utf16(r#"{"netCommandId":"clientPasswordChallenge","coach":"Kalimar"}"#);
        let cmd = decode_text_message(&compressed, true);
        match cmd {
            Some(AnyNetCommand::Client(AnyClientCommand::ClientPasswordChallenge(c))) => {
                assert_eq!(c.coach.as_deref(), Some("Kalimar"));
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn undecodable_messages_are_dropped() {
        assert!(decode_text_message("not json", false).is_none());
        assert!(decode_text_message(r#"{"netCommandId":"noSuchCommand"}"#, false).is_none());
    }

    #[test]
    fn encode_round_trips_through_decompress() {
        let json = r#"{"netCommandId":"serverVersion","commandNr":0}"#;
        let bytes = encode_text_message(json, true).unwrap();
        let text = String::from_utf8(bytes).unwrap();
        assert_eq!(decompress_from_utf16(&text).as_deref(), Some(json));
        assert_eq!(encode_text_message(json, false).unwrap(), json.as_bytes());
    }

    /// Proves the disconnect cleanup — enqueuing `AnyInternalServerCommand::SocketClosed` —
    /// reaches the real `ServerCommandHandlerSocketClosed` via `ServerCommunication`'s dispatch
    /// loop, observed through its `serverLeave` broadcast to a second session.
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
            command_compression: false,
        };

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
