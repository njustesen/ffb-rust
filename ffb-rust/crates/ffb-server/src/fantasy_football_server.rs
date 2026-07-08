/// 1:1 translation of com.fumbbl.ffb.server.FantasyFootballServer.
///
/// Top-level orchestrator: owns `GameCache`, `SessionManager`, and
/// `ServerCommunication`; binds the axum HTTP/WebSocket listener.
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use axum::Router;
use axum::routing::get;
use crate::game_cache::GameCache;
use crate::net::command_socket::{AppState, ws_handler};
use crate::net::server_communication::ServerCommunication;
use crate::net::session_manager::SessionManager;

/// Java: `FantasyFootballServer`
pub struct FantasyFootballServer {
    game_cache: Arc<Mutex<GameCache>>,
    session_manager: Arc<Mutex<SessionManager>>,
}

impl FantasyFootballServer {
    /// Java: `new FantasyFootballServer()`
    pub fn new() -> Self {
        Self {
            game_cache: Arc::new(Mutex::new(GameCache::new())),
            session_manager: Arc::new(Mutex::new(SessionManager::new())),
        }
    }

    /// Java: `start()` — binds the WebSocket listener and runs the event loop.
    pub async fn run(self, addr: SocketAddr) {
        let server_comms = ServerCommunication::new(
            Arc::clone(&self.game_cache),
            Arc::clone(&self.session_manager),
        );

        let state = AppState {
            game_cache: Arc::clone(&self.game_cache),
            session_manager: Arc::clone(&self.session_manager),
            dispatch_tx: server_comms.sender(),
        };

        let app = Router::new()
            .route("/", get(ws_handler))
            .with_state(state);

        log::info!("FFB server listening on ws://{}", addr);
        let listener = tokio::net::TcpListener::bind(addr).await
            .unwrap_or_else(|e| panic!("failed to bind {}: {}", addr, e));
        axum::serve(listener, app).await
            .unwrap_or_else(|e| panic!("server error: {}", e));
    }
}

impl Default for FantasyFootballServer {
    fn default() -> Self { Self::new() }
}
