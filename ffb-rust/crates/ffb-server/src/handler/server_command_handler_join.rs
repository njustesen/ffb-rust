/// 1:1 translation of com.fumbbl.ffb.server.handler.ServerCommandHandlerJoin.
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use ffb_model::enums::{NetCommandId, ServerStatus};
use ffb_model::model::ClientMode;
use ffb_protocol::commands::client_command_join::ClientCommandJoin;
use ffb_protocol::commands::server_command_status::ServerCommandStatus;
use ffb_protocol::server_commands::{GameListEntry, ServerCommand, ServerGameList};
use crate::db::db_connection_manager::DbConnectionManager;
use crate::db::query::db_password_for_coach_query::DbPasswordForCoachQuery;
use crate::game_cache::GameCache;
use crate::model::received_command::{ReceivedCommand, SessionId};
use crate::net::commands::any_internal_server_command::AnyInternalServerCommand;
use crate::net::commands::internal_server_command_join_approved::InternalServerCommandJoinApproved;
use crate::net::session_manager::SessionManager;

/// Java: `ServerCommandHandlerJoin extends ServerCommandHandler`.
pub struct ServerCommandHandlerJoin {
    game_cache: Arc<Mutex<GameCache>>,
    session_manager: Arc<Mutex<SessionManager>>,
    db_connection_manager: Arc<Mutex<DbConnectionManager>>,
    /// Java: `communication.handleCommand(receivedJoinApproved)` — the redispatch sink
    /// this handler hands its `InternalServerCommandJoinApproved` follow-up to. A clone of
    /// the same `mpsc::UnboundedSender<ReceivedCommand>` that feeds
    /// `net::server_communication::dispatch_loop` (see `ServerCommunication::new`), so the
    /// redispatched command lands back on the same single-consumer dispatch queue Java's
    /// `BlockingQueue` models.
    dispatch_tx: mpsc::UnboundedSender<ReceivedCommand>,
}

impl ServerCommandHandlerJoin {
    pub fn new(
        game_cache: Arc<Mutex<GameCache>>,
        session_manager: Arc<Mutex<SessionManager>>,
        db_connection_manager: Arc<Mutex<DbConnectionManager>>,
        dispatch_tx: mpsc::UnboundedSender<ReceivedCommand>,
    ) -> Self {
        Self { game_cache, session_manager, db_connection_manager, dispatch_tx }
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
    ///         DbPasswordForCoachQuery passwordQuery = ...;
    ///         String password = passwordQuery.execute(joinCommand.getCoach());
    ///         if (joinCommand.getPassword().equals(password)) {
    ///             InternalServerCommandJoinApproved joinApprovedCommand = new InternalServerCommandJoinApproved(...);
    ///             communication.handleCommand(new ReceivedCommand(joinApprovedCommand, session));
    ///         } else {
    ///             communication.sendStatus(session, ServerStatus.ERROR_WRONG_PASSWORD, null);
    ///         }
    ///     }
    /// } else {
    ///     GameList gameList = (PLAYER == mode) ? gameCache.findOpenGamesForCoach(coach) : gameCache.findActiveGames();
    ///     communication.sendGameList(session, gameList);
    /// }
    /// return true;
    /// ```
    ///
    /// Java's `ServerMode.FUMBBL` branch (`FumbblRequestCheckAuthorization`, an HTTP
    /// dispatch through `getServer().getRequestProcessor()`) has no `ServerMode`/
    /// request-processor plumbing threaded into this handler — same documented,
    /// separately-scoped gap as `ServerCommandHandlerScheduleGame`'s own FUMBBL branch —
    /// so only the standalone `DbPasswordForCoachQuery` path is translated here.
    pub async fn handle_command(&self, join_command: &ClientCommandJoin, session_id: SessionId) -> bool {
        let has_target = join_command.get_game_id() > 0
            || join_command.get_game_name().map(|n| !n.is_empty()).unwrap_or(false)
            || join_command.get_client_mode() == Some(&ClientMode::REPLAY);

        if has_target {
            let coach = join_command.get_coach().unwrap_or_default().to_string();
            let supplied_password = join_command.get_password().unwrap_or_default().to_string();

            let stored_password = self.query_password_for_coach(&coach).await;

            if stored_password.as_deref() == Some(supplied_password.as_str()) {
                // Java: `Arrays.asList("DEV", "STATE_EDIT")`.
                let join_approved = InternalServerCommandJoinApproved::new(
                    join_command.get_game_id(),
                    join_command.get_game_name().unwrap_or_default().to_string(),
                    coach,
                    join_command.get_team_id().unwrap_or_default().to_string(),
                    join_command.get_client_mode().map(|m| m.get_name().to_string()).unwrap_or_default(),
                    vec!["DEV".to_string(), "STATE_EDIT".to_string()],
                );
                if let Err(e) = self.dispatch_tx.send(ReceivedCommand::new_internal(
                    AnyInternalServerCommand::JoinApproved(join_approved),
                    session_id,
                )) {
                    log::error!("dispatch channel closed, could not redispatch JoinApproved: {}", e);
                }
            } else {
                let status = ServerCommandStatus::new(ServerStatus::ErrorWrongPassword, ServerStatus::ErrorWrongPassword.message());
                self.session_manager.lock().unwrap().send_to(session_id, &status.to_json_value().to_string());
            }

            return true;
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

    /// Java: `DbQueryFactory.getStatement(DbStatementId.PASSWORD_FOR_COACH_QUERY).execute(coach)`.
    /// Degrades to "no stored password" when no DB pool is configured (e.g. in tests),
    /// matching this crate's established `pool_ready()`-gated pattern for DB-backed handlers.
    async fn query_password_for_coach(&self, coach: &str) -> Option<String> {
        let db = self.db_connection_manager.lock().unwrap().clone();
        if !db.pool_ready() {
            return None;
        }
        let mut conn = db.open_db_connection().await.ok()?;
        let result = DbPasswordForCoachQuery::new().execute(&mut conn, coach).await;
        let _ = db.close_db_connection(conn).await;
        result.ok().flatten()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_session() -> (
        Arc<Mutex<GameCache>>,
        Arc<Mutex<SessionManager>>,
        Arc<Mutex<DbConnectionManager>>,
        mpsc::UnboundedReceiver<String>,
    ) {
        let gc = Arc::new(Mutex::new(GameCache::new()));
        let sm = Arc::new(Mutex::new(SessionManager::new()));
        let db = Arc::new(Mutex::new(DbConnectionManager::new()));
        let (tx, rx) = mpsc::unbounded_channel();
        sm.lock().unwrap().add_session(1, 0, "Coach".into(), ClientMode::PLAYER, true, vec![], tx);
        (gc, sm, db, rx)
    }

    fn handler_with_dispatch(
        gc: Arc<Mutex<GameCache>>,
        sm: Arc<Mutex<SessionManager>>,
        db: Arc<Mutex<DbConnectionManager>>,
    ) -> (ServerCommandHandlerJoin, mpsc::UnboundedReceiver<ReceivedCommand>) {
        let (dispatch_tx, dispatch_rx) = mpsc::unbounded_channel();
        (ServerCommandHandlerJoin::new(gc, sm, db, dispatch_tx), dispatch_rx)
    }

    #[test]
    fn construct() {
        let (gc, sm, db, _rx) = setup_session();
        let _ = handler_with_dispatch(gc, sm, db);
    }

    #[tokio::test]
    async fn get_id_is_client_join() {
        let (gc, sm, db, _rx) = setup_session();
        let (handler, _drx) = handler_with_dispatch(gc, sm, db);
        assert_eq!(handler.get_id(), NetCommandId::ClientJoin);
    }

    #[tokio::test]
    async fn lobby_join_sends_game_list_of_known_games() {
        let (gc, sm, db, mut rx) = setup_session();
        gc.lock().unwrap().create_game_state();
        let (handler, _drx) = handler_with_dispatch(gc, sm, db);
        let join = ClientCommandJoin { coach: Some("Coach".into()), ..Default::default() };
        assert!(handler.handle_command(&join, 1).await);
        let msg = rx.try_recv().expect("expected a ServerGameList message");
        assert!(msg.contains("serverGameList"));
    }

    #[tokio::test]
    async fn lobby_join_with_no_games_sends_empty_list() {
        let (gc, sm, db, mut rx) = setup_session();
        let (handler, _drx) = handler_with_dispatch(gc, sm, db);
        let join = ClientCommandJoin::new();
        assert!(handler.handle_command(&join, 1).await);
        let msg = rx.try_recv().expect("expected a ServerGameList message");
        assert!(msg.contains("\"games\":[]"));
    }

    /// Without a DB pool configured (the test default), `query_password_for_coach` always
    /// returns `None`, so any supplied password fails the equality check — the wrong-password
    /// status is sent, not a redispatch.
    #[tokio::test]
    async fn targeted_join_without_db_pool_sends_wrong_password_status() {
        let (gc, sm, db, mut rx) = setup_session();
        let (handler, mut drx) = handler_with_dispatch(gc, sm, db);
        let join = ClientCommandJoin { game_id: 5, coach: Some("Coach".into()), password: Some("test".into()), ..Default::default() };
        assert!(handler.handle_command(&join, 1).await);
        let msg = rx.try_recv().expect("expected a ServerCommandStatus message");
        assert!(msg.contains("Wrong Password"));
        assert!(drx.try_recv().is_err(), "no redispatch should have happened");
    }

    /// A live DB pool is required for `query_password_for_coach` to ever return `Some(..)`
    /// (see `db_connection_manager.rs`'s `pool_ready()` gate); without one, every targeted
    /// join fails the password check, regardless of the supplied password — including an
    /// empty one. `DbPasswordForCoachQuery::execute`'s own SQL-shape coverage lives in
    /// `db/query/db_password_for_coach_query.rs`; this test only proves the wiring reaches
    /// the wrong-password path when no pool is configured.
    #[tokio::test]
    async fn targeted_join_with_empty_password_and_no_db_pool_still_fails() {
        let (gc, sm, db, mut rx) = setup_session();
        let (handler, mut drx) = handler_with_dispatch(gc, sm, db);
        let join = ClientCommandJoin { game_id: 5, coach: Some("Coach".into()), password: Some(String::new()), ..Default::default() };
        assert!(handler.handle_command(&join, 1).await);
        let msg = rx.try_recv().expect("expected a ServerCommandStatus message");
        assert!(msg.contains("Wrong Password"));
        assert!(drx.try_recv().is_err(), "no redispatch should have happened without a DB pool");
    }

    #[tokio::test]
    async fn targeted_join_by_game_name_is_also_treated_as_targeted() {
        let (gc, sm, db, mut rx) = setup_session();
        let (handler, mut drx) = handler_with_dispatch(gc, sm, db);
        let join = ClientCommandJoin { game_name: Some("SomeGame".into()), coach: Some("Coach".into()), password: Some(String::new()), ..Default::default() };
        assert!(handler.handle_command(&join, 1).await);
        // Targeted-by-name is still gated by the same (unconfigured) password check, so it
        // reaches the wrong-password status path rather than the lobby game list.
        let msg = rx.try_recv().expect("expected a ServerCommandStatus message, not a lobby game list");
        assert!(msg.contains("Wrong Password"));
        assert!(drx.try_recv().is_err());
    }

    #[tokio::test]
    async fn targeted_join_by_replay_mode_is_also_treated_as_targeted() {
        let (gc, sm, db, mut rx) = setup_session();
        let (handler, mut drx) = handler_with_dispatch(gc, sm, db);
        let join = ClientCommandJoin { client_mode: Some(ClientMode::REPLAY), coach: Some("Coach".into()), password: Some(String::new()), ..Default::default() };
        assert!(handler.handle_command(&join, 1).await);
        let msg = rx.try_recv().expect("expected a ServerCommandStatus message, not a lobby game list");
        assert!(msg.contains("Wrong Password"));
        assert!(drx.try_recv().is_err());
    }
}
