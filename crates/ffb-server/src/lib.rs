pub mod api;
pub mod ws;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};

use api::AppState;

pub fn create_app() -> Router {
    let state = Arc::new(Mutex::new(HashMap::new() as AppState));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/", get(api::index_handler))
        .route("/game/new", post(api::new_game))
        .route("/game/{id}/state", get(api::get_state))
        .route("/game/{id}/action", post(api::post_action))
        .route("/game/{id}/ws", get(ws::ws_handler))
        .layer(cors)
        .with_state(state)
}
