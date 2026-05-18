use std::time::Duration;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::Response,
};
use tokio::time::interval;

use crate::api::{build_board_state, SharedState};

/// WebSocket upgrade handler — streams state updates to the client.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(shared): State<SharedState>,
    Path(id): Path<String>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, shared, id))
}

async fn handle_socket(mut socket: WebSocket, shared: SharedState, game_id: String) {
    // Poll the game state every 500ms and push if it changed.
    // This is simpler than a proper pub/sub and sufficient for human vs AI.
    let mut ticker = interval(Duration::from_millis(500));
    let mut last_json = String::new();

    loop {
        ticker.tick().await;

        // Check if socket closed
        // (we don't block on recv, just poll state)
        let json = {
            let map = shared.lock().unwrap();
            if let Some(session) = map.get(&game_id) {
                let board = build_board_state(&game_id, &session.state, session.human_team);
                serde_json::to_string(&board).unwrap_or_default()
            } else {
                break; // game gone
            }
        };

        if json != last_json {
            last_json = json.clone();
            if socket.send(Message::Text(json)).await.is_err() {
                break; // client disconnected
            }
        }
    }
}
