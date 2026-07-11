/// 1:1 translation of com.fumbbl.ffb.server.handler.ServerCommandHandlerJoin.
use std::sync::{Arc, Mutex};
use ffb_model::enums::NetCommandId;
use ffb_model::model::ClientMode;
use ffb_protocol::commands::client_command_join::ClientCommandJoin;
use ffb_protocol::server_commands::{GameListEntry, ServerCommand, ServerGameList};
use crate::game_cache::GameCache;
use crate::model::received_command::SessionId;
use crate::net::session_manager::SessionManager;

/// Java: `ServerCommandHandlerJoin extends ServerCommandHandler`.
pub struct ServerCommandHandlerJoin {
    game_cache: Arc<Mutex<GameCache>>,
    session_manager: Arc<Mutex<SessionManager>>,
}

impl ServerCommandHandlerJoin {
    pub fn new(game_cache: Arc<Mutex<GameCache>>, session_manager: Arc<Mutex<SessionManager>>) -> Self {
        Self { game_cache, session_manager }
    }

    /// Java: `getId()` — returns `NetCommandId.CLIENT_JOIN`.
    pub fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientJoin
    }

    /// Java: `handleCommand(ReceivedCommand)`.
    ///
    /// ```java
    /// if ((joinCommand.getGameId() > 0) || StringTool.isProvided(joinCommand.getGameName()) || joinCommand.getClientMode() == ClientMode.REPLAY) {
    ///     if (ServerMode.FUMBBL == getServer().getMode()) {
    ///         getServer().getRequestProcessor().add(new FumbblRequestCheckAuthorization(...));
    ///     } else {
    ///         // standalone: password check via DbPasswordForCoachQuery, then
    ///         // dispatch an InternalServerCommandJoinApproved or ERROR_WRONG_PASSWORD.
    ///     }
    /// } else {
    ///     GameList gameList = (PLAYER == mode) ? gameCache.findOpenGamesForCoach(coach) : gameCache.findActiveGames();
    ///     communication.sendGameList(session, gameList);
    /// }
    /// return true;
    /// ```
    ///
    /// Targeted-join (by game id / name / replay) requires either the FUMBBL
    /// authorization HTTP pipeline or the standalone `DbPasswordForCoachQuery`
    /// — neither is wired in the Rust server yet, so that branch is a narrow
    /// `todo!()`. The lobby-listing branch is fully implemented against the
    /// in-memory `GameCache`.
    pub fn handle_command(&self, join_command: &ClientCommandJoin, session_id: SessionId) -> bool {
        let has_target = join_command.get_game_id() > 0
            || join_command.get_game_name().map(|n| !n.is_empty()).unwrap_or(false)
            || join_command.get_client_mode() == Some(&ClientMode::REPLAY);

        if has_target {
            // Java: FumbblRequestCheckAuthorization (HTTP) or DbPasswordForCoachQuery (DB).
            todo!("Phase ZV: needs FumbblRequestCheckAuthorization / DbPasswordForCoachQuery wiring");
        }

        // Java: `gameCache.findOpenGamesForCoach` / `findActiveGames` — the Rust
        // in-memory GameCache has no per-coach/status filtering yet, so every
        // known game id is listed (best-effort stand-in for `findActiveGames`).
        let games: Vec<GameListEntry> = {
            let gc = self.game_cache.lock().unwrap();
            gc.all_game_ids()
                .into_iter()
                .map(|id| GameListEntry {
                    game_id: id.to_string(),
                    home_team: String::new(),
                    away_team: String::new(),
                    status: String::new(),
                })
                .collect()
        };
        let command = ServerCommand::ServerGameList(ServerGameList { games });
        if let Ok(json) = serde_json::to_string(&command) {
            self.session_manager.lock().unwrap().send_to(session_id, &json);
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    fn setup_session() -> (Arc<Mutex<GameCache>>, Arc<Mutex<SessionManager>>, mpsc::UnboundedReceiver<String>) {
        let gc = Arc::new(Mutex::new(GameCache::new()));
        let sm = Arc::new(Mutex::new(SessionManager::new()));
        let (tx, rx) = mpsc::unbounded_channel();
        sm.lock().unwrap().add_session(1, 0, "Coach".into(), ClientMode::PLAYER, true, vec![], tx);
        (gc, sm, rx)
    }

    #[test]
    fn construct() {
        let (gc, sm, _rx) = setup_session();
        let _ = ServerCommandHandlerJoin::new(gc, sm);
    }

    #[test]
    fn get_id_is_client_join() {
        let (gc, sm, _rx) = setup_session();
        let handler = ServerCommandHandlerJoin::new(gc, sm);
        assert_eq!(handler.get_id(), NetCommandId::ClientJoin);
    }

    #[test]
    fn lobby_join_sends_game_list_of_known_games() {
        let (gc, sm, mut rx) = setup_session();
        gc.lock().unwrap().create_game_state();
        let handler = ServerCommandHandlerJoin::new(gc, sm);
        let join = ClientCommandJoin { coach: Some("Coach".into()), ..Default::default() };
        assert!(handler.handle_command(&join, 1));
        let msg = rx.try_recv().expect("expected a ServerGameList message");
        assert!(msg.contains("serverGameList"));
    }

    #[test]
    fn lobby_join_with_no_games_sends_empty_list() {
        let (gc, sm, mut rx) = setup_session();
        let handler = ServerCommandHandlerJoin::new(gc, sm);
        let join = ClientCommandJoin::new();
        assert!(handler.handle_command(&join, 1));
        let msg = rx.try_recv().expect("expected a ServerGameList message");
        assert!(msg.contains("\"games\":[]"));
    }

    #[test]
    fn targeted_join_by_game_id_hits_auth_stub() {
        let (gc, sm, _rx) = setup_session();
        let handler = ServerCommandHandlerJoin::new(gc, sm);
        let join = ClientCommandJoin { game_id: 5, ..Default::default() };
        let result = std::panic::catch_unwind(|| handler.handle_command(&join, 1));
        assert!(result.is_err());
    }
}
