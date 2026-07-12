/// 1:1 translation of com.fumbbl.ffb.server.util.MarkerLoadingService.
///
/// Java:
/// ```java
/// public void loadMarker(GameState gameState, Session session, boolean homeTeam, boolean auto, SortMode sortMode) {
///     if (auto) {
///         gameState.getServer().getRequestProcessor().add(new FumbblRequestLoadPlayerMarkings(gameState, session, sortMode));
///     } else {
///         IDbStatementFactory statementFactory = gameState.getServer().getDbQueryFactory();
///         DbPlayerMarkersQuery dbPlayerMarkersQuery = (DbPlayerMarkersQuery) statementFactory.getStatement(DbStatementId.PLAYER_MARKERS_QUERY);
///         dbPlayerMarkersQuery.execute(gameState, homeTeam);
///     }
/// }
/// ```
///
/// Java reaches its dependencies via `gameState.getServer().getX()`; per this crate's
/// convention (see `game_cache.rs`, `server_command_handler_password_challenge.rs`) they
/// are threaded through explicitly instead: a `ServerRequestProcessor` handle for the
/// `auto` branch, and a live `mysql_async::Conn` + `team_id` for the DB branch (this
/// crate's `DbPlayerMarkersQuery::execute` takes `team_id` directly rather than
/// `GameState`+`homeTeam`, since there is no `GameState`/`FieldModel` wiring at the DB
/// layer yet — see that file's own doc comment).
///
/// Java's `FumbblRequestLoadPlayerMarkings(gameState, session, sortMode)` constructor
/// pulls the coach name and game id off of `gameState`/`session` to build its request URL
/// and, on completion, applies the result to the session's marking config and dispatches
/// `InternalServerCommandApplyAutomatedPlayerMarkings` — none of that session/command
/// plumbing exists in this crate yet (same documented gap as
/// `ServerCommandHandlerLoadAutomaticPlayerMarkings`). The `coach` and `sort_mode` are
/// threaded through explicitly here instead of derived from a `GameState`/`Session`.
use std::sync::{Arc, Mutex};

use tokio::sync::mpsc;

use ffb_model::marking::sort_mode::SortMode;
use mysql_async::{Conn, Error as DbError};

use crate::db::query::db_player_markers_query::DbPlayerMarkersQuery;
use crate::model::received_command::{ReceivedCommand, SessionId};
use crate::net::commands::any_internal_server_command::AnyInternalServerCommand;
use crate::net::commands::internal_server_command_apply_automated_player_markings::InternalServerCommandApplyAutomatedPlayerMarkings;
use crate::request::fumbbl::util_fumbbl_request::HttpClient;
use crate::request::server_request::ServerRequest;
use crate::request::server_request_processor::ServerRequestProcessor;

/// Destination for the real `InternalServerCommandApplyAutomatedPlayerMarkings` redispatch —
/// `game_id` matches Java's `gameState.getId()`, threaded through explicitly (this crate's
/// established convention) rather than derived from a `GameState`/`Session` reference.
pub struct MarkerDispatch {
    pub dispatch_tx: mpsc::UnboundedSender<ReceivedCommand>,
    pub session_id: SessionId,
    pub game_id: i64,
}

/// `ServerRequest` adapter around `FumbblRequestLoadPlayerMarkings.process` — matching the
/// pattern used by `QueuedLoadAutomaticPlayerMarkingsRequest` in
/// `handler/server_command_handler_load_automatic_player_markings.rs`. Java applies the
/// fetched config to the session's marking config and dispatches
/// `InternalServerCommandApplyAutomatedPlayerMarkings`; the latter is real here (see
/// `dispatch`), but `load_marker_auto`'s only current caller
/// (`util/server_start_game.rs::send_server_join`) does not yet thread a game id/dispatch
/// channel through its own call chain (`MarkerContext`), so `dispatch` is `None` there today
/// — a narrower, separately-scoped wiring gap than "the redispatch doesn't exist."
struct QueuedLoadPlayerMarkingsRequest {
    request: Mutex<crate::request::fumbbl::fumbbl_request_load_player_markings::FumbblRequestLoadPlayerMarkings>,
    client: Arc<dyn HttpClient + Send + Sync>,
    markings_url_template: String,
    coach: String,
    /// Java: `sortMode` — applied to the session's marking config, which this crate has no
    /// equivalent for yet (same gap as the rest of this struct's session plumbing).
    #[allow(dead_code)]
    sort_mode: SortMode,
    dispatch: Option<MarkerDispatch>,
}

impl ServerRequest for QueuedLoadPlayerMarkingsRequest {
    fn process(&self) -> Result<(), String> {
        let config = {
            let mut request = self.request.lock().unwrap();
            request.process(self.client.as_ref(), &self.markings_url_template, &self.coach)?
        };
        if let (Some(config), Some(dispatch)) = (config, &self.dispatch) {
            let cmd = InternalServerCommandApplyAutomatedPlayerMarkings::new(config, dispatch.game_id);
            let _ = dispatch.dispatch_tx.send(ReceivedCommand::new_internal(
                AnyInternalServerCommand::ApplyAutomatedPlayerMarkings(cmd),
                dispatch.session_id,
            ));
        }
        Ok(())
    }

    fn get_request_url(&self) -> &str {
        // Locking to read a `&str` behind a `Mutex` isn't expressible without leaking; the
        // request URL is only meaningfully read after `process()`, so this mirrors
        // `ServerRequest`'s other adapters' best-effort behavior for the pre-process case.
        ""
    }

    fn set_request_url(&mut self, _url: String) {}
}

pub struct MarkerLoadingService;

impl MarkerLoadingService {
    pub fn new() -> Self {
        Self
    }

    /// Java: `loadMarker(GameState, Session, boolean, boolean, SortMode)` — `auto` branch.
    /// Enqueues a `FumbblRequestLoadPlayerMarkings` onto the `ServerRequestProcessor`.
    pub fn load_marker_auto(
        &self,
        request_processor: &Arc<Mutex<ServerRequestProcessor>>,
        client: Arc<dyn HttpClient + Send + Sync>,
        markings_url_template: impl Into<String>,
        coach: impl Into<String>,
        sort_mode: SortMode,
        dispatch: Option<MarkerDispatch>,
    ) -> bool {
        let request = Box::new(QueuedLoadPlayerMarkingsRequest {
            request: Mutex::new(crate::request::fumbbl::fumbbl_request_load_player_markings::FumbblRequestLoadPlayerMarkings::new()),
            client,
            markings_url_template: markings_url_template.into(),
            coach: coach.into(),
            sort_mode,
            dispatch,
        });
        request_processor.lock().unwrap().add(request)
    }

    /// Java: `loadMarker(GameState, Session, boolean, boolean, SortMode)` — non-`auto`
    /// branch. Executes `DbPlayerMarkersQuery` directly against `team_id`.
    pub async fn load_marker_from_db(
        &self,
        conn: &mut Conn,
        team_id: &str,
    ) -> Result<Vec<(String, String)>, DbError> {
        DbPlayerMarkersQuery::new().execute(conn, team_id).await
    }
}

impl Default for MarkerLoadingService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::commands::internal_server_command::InternalServerCommand;
    use crate::request::fumbbl::util_fumbbl_request::MockHttpClient;

    #[test]
    fn new_and_default_construct() {
        let _a = MarkerLoadingService::new();
        let _b = MarkerLoadingService::default();
    }

    #[test]
    fn load_marker_auto_enqueues_a_request() {
        let service = MarkerLoadingService::new();
        let processor = Arc::new(Mutex::new(ServerRequestProcessor::new()));
        let client: Arc<dyn HttpClient + Send + Sync> = Arc::new(MockHttpClient { response: Ok("{}".to_string()) });

        let enqueued = service.load_marker_auto(
            &processor,
            client,
            "http://fumbbl/markings/$1",
            "Kalimar",
            SortMode::Default,
            None,
        );

        assert!(enqueued);
        assert_eq!(processor.lock().unwrap().queue_len(), 1);
    }

    #[test]
    fn load_marker_auto_returns_false_when_processor_stopped() {
        let service = MarkerLoadingService::new();
        let processor = Arc::new(Mutex::new(ServerRequestProcessor::new()));
        processor.lock().unwrap().shutdown();
        let client: Arc<dyn HttpClient + Send + Sync> = Arc::new(MockHttpClient { response: Ok("{}".to_string()) });

        let enqueued = service.load_marker_auto(
            &processor,
            client,
            "http://fumbbl/markings/$1",
            "Kalimar",
            SortMode::Default,
            None,
        );

        assert!(!enqueued);
    }

    #[test]
    fn load_marker_auto_with_dispatch_redispatches_apply_automated_player_markings() {
        let service = MarkerLoadingService::new();
        let processor = Arc::new(Mutex::new(ServerRequestProcessor::new()));
        let client: Arc<dyn HttpClient + Send + Sync> = Arc::new(MockHttpClient {
            response: Ok(r#"{"autoMarkingSeparator":"-","autoMarkingRecords":[]}"#.to_string()),
        });
        let (dispatch_tx, mut dispatch_rx) = mpsc::unbounded_channel();

        let enqueued = service.load_marker_auto(
            &processor,
            client,
            "http://fumbbl/markings/$1",
            "Kalimar",
            SortMode::Default,
            Some(MarkerDispatch { dispatch_tx, session_id: 3, game_id: 42 }),
        );
        assert!(enqueued);
        assert!(processor.lock().unwrap().run().is_ok());

        let received = dispatch_rx.try_recv().expect("expected a redispatched ApplyAutomatedPlayerMarkings command");
        assert_eq!(received.session_id, 3);
        match received.command {
            crate::model::received_command::ReceivedNetCommand::Internal(
                AnyInternalServerCommand::ApplyAutomatedPlayerMarkings(cmd),
            ) => {
                assert_eq!(cmd.get_game_id(), 42);
                assert_eq!(cmd.get_auto_marking_config().get_separator(), "-");
            }
            _ => panic!("expected an internal ApplyAutomatedPlayerMarkings command"),
        }
    }
}
