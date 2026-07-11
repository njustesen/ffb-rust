/// 1:1 translation of com.fumbbl.ffb.server.handler.ServerCommandHandlerUpdatePlayerMarkings.
use std::sync::{Arc, Mutex};
use ffb_model::enums::NetCommandId;
use ffb_model::marking::player_marker::PlayerMarker;
use ffb_model::model::game::Game;
use ffb_model::model::ClientMode;
use ffb_model::util::string_tool::is_provided;
use ffb_protocol::commands::client_command_update_player_markings::ClientCommandUpdatePlayerMarkings;
use crate::db::db_connection_manager::DbConnectionManager;
use crate::db::query::db_player_markers_query::DbPlayerMarkersQuery;
use crate::game_cache::GameCache;
use crate::handler::server_command_handler_apply_automated_player_markings::send_update_local_player_markers;
use crate::model::received_command::SessionId;
use crate::net::session_manager::SessionManager;
use crate::request::fumbbl::fumbbl_request_load_player_markings::FumbblRequestLoadPlayerMarkings;
use crate::request::fumbbl::util_fumbbl_request::HttpClient;
use crate::request::server_request::ServerRequest;
use crate::request::server_request_processor::ServerRequestProcessor;

/// `ServerRequest` adapter around [`FumbblRequestLoadPlayerMarkings`] (Java:
/// `new FumbblRequestLoadPlayerMarkings(gameState, session, sortMode)` enqueued on the
/// `ServerRequestProcessor`). Performs the real HTTP fetch; Java's onward
/// `InternalServerCommandApplyAutomatedPlayerMarkings` dispatch has no session/command
/// plumbing in this crate yet, so the fetched config is discarded after the request runs —
/// mirrors `QueuedLoadAutomaticPlayerMarkingsRequest` in the sibling
/// `server_command_handler_load_automatic_player_markings.rs`.
struct QueuedLoadPlayerMarkingsRequest {
    coach: String,
    client: Arc<dyn HttpClient + Send + Sync>,
    markings_url_template: String,
}

impl ServerRequest for QueuedLoadPlayerMarkingsRequest {
    fn process(&self) -> Result<(), String> {
        let mut request = FumbblRequestLoadPlayerMarkings::new();
        request.process(self.client.as_ref(), &self.markings_url_template, &self.coach)?;
        Ok(())
    }

    fn get_request_url(&self) -> &str {
        ""
    }

    fn set_request_url(&mut self, _url: String) {}
}

pub struct ServerCommandHandlerUpdatePlayerMarkings {
    db_connection_manager: Arc<Mutex<DbConnectionManager>>,
    request_processor: Arc<Mutex<ServerRequestProcessor>>,
    client: Arc<dyn HttpClient + Send + Sync>,
    markings_url_template: String,
}

impl ServerCommandHandlerUpdatePlayerMarkings {
    pub fn new(
        db_connection_manager: Arc<Mutex<DbConnectionManager>>,
        request_processor: Arc<Mutex<ServerRequestProcessor>>,
        client: Arc<dyn HttpClient + Send + Sync>,
        markings_url_template: impl Into<String>,
    ) -> Self {
        Self {
            db_connection_manager,
            request_processor,
            client,
            markings_url_template: markings_url_template.into(),
        }
    }

    /// Java: getId() — returns NetCommandId for UPDATE_PLAYER_MARKINGS.
    pub fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientUpdatePlayerMarkings
    }

    fn enqueue_load_player_markings(&self, coach: String) {
        let queued = QueuedLoadPlayerMarkingsRequest {
            coach,
            client: Arc::clone(&self.client),
            markings_url_template: self.markings_url_template.clone(),
        };
        self.request_processor.lock().unwrap().add(Box::new(queued));
    }

    /// Java: handleCommand(ReceivedCommand) — handles updating player markings.
    ///
    /// Takes `Arc<Mutex<..>>` rather than plain references so every lock acquired here is
    /// dropped before crossing an `.await` point (`std::sync::MutexGuard` is `!Send`, and this
    /// method is reachable from `ServerCommandHandlerFactory::handle_command`, which runs
    /// inside `tokio::spawn(dispatch_loop(...))` — same fix shape as
    /// `ServerCommandHandlerDeleteGame::handle_command`'s own `DbConnectionManager` clone).
    pub async fn handle_command(
        &self,
        command: &ClientCommandUpdatePlayerMarkings,
        session_id: SessionId,
        game_cache: &Arc<Mutex<GameCache>>,
        session_manager: &Arc<Mutex<SessionManager>>,
    ) -> bool {
        // Java: long gameId = sessionManager.getGameIdForSession(session);
        //       GameState gameState = getGameCache().getGameStateById(gameId);
        //       if (gameState == null) return false;
        // Java: ClientMode mode = sessionManager.getModeForSession(session);
        // Java: boolean isHome = UtilServerSteps.checkCommandIsFromHomePlayer(gameState, receivedCommand);
        // `checkCommandIsFromHomePlayer` ultimately compares the sending session against the
        // registered home coach session for the game — exactly what
        // `SessionManager::get_session_of_home_coach` already provides.
        let (game_id, mode, is_home) = {
            let sm = session_manager.lock().unwrap();
            let game_id = sm.get_game_id_for_session(session_id);
            (game_id, sm.get_mode_for_session(session_id), sm.get_session_of_home_coach(game_id) == Some(session_id))
        };
        {
            let gc = game_cache.lock().unwrap();
            if gc.get_game_state_by_id(game_id).is_none() {
                return false;
            }
        }

        // Java: if (!commandUpdatePlayerMarkings.isAuto()) sessionManager.removeAutoMarking(session);
        // The Rust `SessionManager` tracks no per-session "auto marking" flag, so there is no
        // such state to clear here.

        match mode {
            Some(ClientMode::PLAYER) => {
                // Java: new MarkerLoadingService().loadMarker(gameState, session, isHome,
                //     commandUpdatePlayerMarkings.isAuto(), commandUpdatePlayerMarkings.getSortMode());
                if command.is_auto() {
                    let coach = {
                        session_manager.lock().unwrap().get_coach_for_session(session_id).unwrap_or_default().to_string()
                    };
                    self.enqueue_load_player_markings(coach);
                } else {
                    // Java: `DbPlayerMarkersQuery.execute(gameState, isHome)` — clears the
                    // requesting side's marker text and repopulates it from the DB.
                    let manager = self.db_connection_manager.lock().unwrap().clone();
                    if manager.pool_ready() {
                        if let Ok(mut conn) = manager.open_db_connection().await {
                            let team_id = {
                                let mut gc = game_cache.lock().unwrap();
                                gc.get_game_state_by_id_mut(game_id).and_then(|gs| gs.get_game_mut()).map(|game| {
                                    if is_home { game.team_home.id.clone() } else { game.team_away.id.clone() }
                                })
                            };
                            if let Some(team_id) = team_id {
                                if !team_id.is_empty() {
                                    if let Ok(rows) =
                                        DbPlayerMarkersQuery::new().execute(&mut conn, &team_id).await
                                    {
                                        let mut gc = game_cache.lock().unwrap();
                                        if let Some(game_state) = gc.get_game_state_by_id_mut(game_id) {
                                            if let Some(game) = game_state.get_game_mut() {
                                                apply_player_markers(game, is_home, rows);
                                            }
                                        }
                                    }
                                }
                            }
                            let _ = manager.close_db_connection(conn).await;
                        }
                    }
                }
            }
            Some(ClientMode::SPECTATOR) => {
                if command.is_auto() {
                    // Java: getRequestProcessor().add(new FumbblRequestLoadPlayerMarkings(gameState, session, sortMode));
                    let coach = {
                        session_manager.lock().unwrap().get_coach_for_session(session_id).unwrap_or_default().to_string()
                    };
                    self.enqueue_load_player_markings(coach);
                } else {
                    // Java: getCommunication().sendUpdateLocalPlayerMarkers(session, Collections.emptyList());
                    let sm = session_manager.lock().unwrap();
                    send_update_local_player_markers(&sm, session_id, &[]);
                }
            }
            _ => {}
        }

        true
    }
}

/// Java: `DbPlayerMarkersQuery.queryMarkers(GameState, boolean)` — the model-mutation tail of
/// the query, run here against the rows already fetched by `DbPlayerMarkersQuery::execute`.
/// First blanks the requesting side's text on every existing marker (Java re-adds each marker
/// with `setHomeText("")`/`setAwayText("")` before applying fresh rows), then applies each
/// `(player_id, text)` row that refers to a real player and has non-empty text.
fn apply_player_markers(game: &mut Game, home_team: bool, rows: Vec<(String, String)>) {
    let existing: Vec<PlayerMarker> = game.field_model.get_player_markers().to_vec();
    for mut marker in existing {
        if home_team {
            marker.set_home_text("");
        } else {
            marker.set_away_text("");
        }
        game.field_model.add_player_marker(marker);
    }

    for (player_id, text) in rows {
        if game.player(&player_id).is_some() && is_provided(Some(text.as_str())) {
            let mut marker = game
                .field_model
                .get_player_marker(&player_id)
                .cloned()
                .unwrap_or_else(|| PlayerMarker::with_player_id(player_id.clone()));
            if home_team {
                marker.set_home_text(text);
            } else {
                marker.set_away_text(text);
            }
            game.field_model.add_player_marker(marker);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::request::fumbbl::util_fumbbl_request::MockHttpClient;
    use tokio::sync::mpsc;

    fn handler() -> ServerCommandHandlerUpdatePlayerMarkings {
        ServerCommandHandlerUpdatePlayerMarkings::new(
            Arc::new(Mutex::new(DbConnectionManager::new())),
            Arc::new(Mutex::new(ServerRequestProcessor::new())),
            Arc::new(MockHttpClient { response: Ok("{}".to_string()) }),
            "http://fumbbl/markings/$1",
        )
    }

    #[test]
    fn construct() {
        let _ = handler();
    }

    #[test]
    fn get_id_is_update_player_markings() {
        let h = handler();
        assert_eq!(h.get_id(), NetCommandId::ClientUpdatePlayerMarkings);
    }

    #[tokio::test]
    async fn handle_command_missing_gamestate_returns_false() {
        let h = handler();
        let cache = Arc::new(Mutex::new(GameCache::new()));
        let sm = Arc::new(Mutex::new(SessionManager::new()));
        let command = ClientCommandUpdatePlayerMarkings { entropy: None, auto: false, sort_mode_name: None };
        // No session registered => get_game_id_for_session returns 0 => no gamestate found.
        assert!(!h.handle_command(&command, 1, &cache, &sm).await);
    }

    #[tokio::test]
    async fn handle_command_spectator_non_auto_sends_empty_markers() {
        let h = handler();
        let cache = Arc::new(Mutex::new(GameCache::new()));
        let game_id = cache.lock().unwrap().create_game_state();

        let sm = Arc::new(Mutex::new(SessionManager::new()));
        let (tx, mut rx) = mpsc::unbounded_channel();
        sm.lock().unwrap().add_session(1, game_id, "Spec".into(), ClientMode::SPECTATOR, false, vec![], tx);

        let command = ClientCommandUpdatePlayerMarkings { entropy: None, auto: false, sort_mode_name: None };
        assert!(h.handle_command(&command, 1, &cache, &sm).await);

        let sent = rx.try_recv().expect("expected an empty markers message");
        assert!(sent.contains("\"markers\":[]"));
    }

    #[tokio::test]
    async fn handle_command_spectator_auto_enqueues_a_request() {
        let h = handler();
        let cache = Arc::new(Mutex::new(GameCache::new()));
        let game_id = cache.lock().unwrap().create_game_state();

        let sm = Arc::new(Mutex::new(SessionManager::new()));
        let (tx, _rx) = mpsc::unbounded_channel();
        sm.lock().unwrap().add_session(1, game_id, "Spec".into(), ClientMode::SPECTATOR, false, vec![], tx);

        let command = ClientCommandUpdatePlayerMarkings { entropy: None, auto: true, sort_mode_name: None };
        assert!(h.handle_command(&command, 1, &cache, &sm).await);
        assert_eq!(h.request_processor.lock().unwrap().queue_len(), 1);
    }

    #[tokio::test]
    async fn handle_command_unregistered_mode_is_a_noop_returning_true() {
        let h = handler();
        let cache = Arc::new(Mutex::new(GameCache::new()));
        let game_id = cache.lock().unwrap().create_game_state();

        let sm = Arc::new(Mutex::new(SessionManager::new()));
        let (tx, _rx) = mpsc::unbounded_channel();
        // REPLAY mode has no handling branch in Java either — falls through to `return true`.
        sm.lock().unwrap().add_session(1, game_id, "Replay".into(), ClientMode::REPLAY, false, vec![], tx);

        let command = ClientCommandUpdatePlayerMarkings { entropy: None, auto: true, sort_mode_name: None };
        assert!(h.handle_command(&command, 1, &cache, &sm).await);
    }

    #[tokio::test]
    async fn is_home_matches_session_of_home_coach() {
        let cache = Arc::new(Mutex::new(GameCache::new()));
        let game_id = cache.lock().unwrap().create_game_state();
        let sm = Arc::new(Mutex::new(SessionManager::new()));
        let (tx1, _rx1) = mpsc::unbounded_channel();
        let (tx2, mut rx2) = mpsc::unbounded_channel();
        sm.lock().unwrap().add_session(1, game_id, "Home".into(), ClientMode::PLAYER, true, vec![], tx1);
        sm.lock().unwrap().add_session(2, game_id, "AwaySpec".into(), ClientMode::SPECTATOR, false, vec![], tx2);

        let h = handler();
        let command = ClientCommandUpdatePlayerMarkings { entropy: None, auto: false, sort_mode_name: None };
        assert!(h.handle_command(&command, 2, &cache, &sm).await);
        let sent = rx2.try_recv().expect("spectator branch should send markers");
        assert!(sent.contains("\"markers\":[]"));
    }

    /// Without a live DB pool configured, the PLAYER + non-auto branch's DB step is skipped
    /// (`pool_ready()` is false) — the handler still returns `true` without panicking.
    #[tokio::test]
    async fn handle_command_player_non_auto_without_db_pool_is_a_noop_returning_true() {
        let h = handler();
        let cache = Arc::new(Mutex::new(GameCache::new()));
        let game_id = cache.lock().unwrap().create_game_state();
        let sm = Arc::new(Mutex::new(SessionManager::new()));
        let (tx, _rx) = mpsc::unbounded_channel();
        sm.lock().unwrap().add_session(1, game_id, "Home".into(), ClientMode::PLAYER, true, vec![], tx);

        let command = ClientCommandUpdatePlayerMarkings { entropy: None, auto: false, sort_mode_name: None };
        assert!(h.handle_command(&command, 1, &cache, &sm).await);
    }

    #[tokio::test]
    async fn handle_command_player_auto_enqueues_a_request() {
        let h = handler();
        let cache = Arc::new(Mutex::new(GameCache::new()));
        let game_id = cache.lock().unwrap().create_game_state();
        let sm = Arc::new(Mutex::new(SessionManager::new()));
        let (tx, _rx) = mpsc::unbounded_channel();
        sm.lock().unwrap().add_session(1, game_id, "Home".into(), ClientMode::PLAYER, true, vec![], tx);

        let command = ClientCommandUpdatePlayerMarkings { entropy: None, auto: true, sort_mode_name: None };
        assert!(h.handle_command(&command, 1, &cache, &sm).await);
        assert_eq!(h.request_processor.lock().unwrap().queue_len(), 1);
    }

    #[test]
    fn apply_player_markers_populates_home_text_for_known_player() {
        use ffb_model::model::team::Team;
        let home = Team {
            id: "t1".into(),
            name: "t1".into(),
            race: "Human".into(),
            roster_id: "human".into(),
            coach: "coach".into(),
            rerolls: 0,
            apothecaries: 0,
            bribes: 0,
            master_chefs: 0,
            prayers_to_nuffle: 0,
            bloodweiser_kegs: 0,
            riotous_rookies: 0,
            cheerleaders: 0,
            assistant_coaches: 0,
            fan_factor: 0,
            dedicated_fans: 0,
            team_value: 0,
            treasury: 0,
            special_rules: vec![],
            players: vec![ffb_model::model::player::Player { id: "p1".into(), ..Default::default() }],
            vampire_lord: false,
            necromancer: false,
        };
        let away = Team { id: "t2".into(), players: vec![], ..home.clone() };
        let mut game = Game::new(home, away, ffb_model::enums::Rules::Bb2025);

        apply_player_markers(&mut game, true, vec![("p1".to_string(), "Nice job".to_string())]);

        let marker = game.field_model.get_player_marker("p1").unwrap();
        assert_eq!(marker.get_home_text(), Some("Nice job"));
    }

    #[test]
    fn apply_player_markers_ignores_unknown_player() {
        use ffb_model::model::team::Team;
        let home = Team {
            id: "t1".into(),
            name: "t1".into(),
            race: "Human".into(),
            roster_id: "human".into(),
            coach: "coach".into(),
            rerolls: 0,
            apothecaries: 0,
            bribes: 0,
            master_chefs: 0,
            prayers_to_nuffle: 0,
            bloodweiser_kegs: 0,
            riotous_rookies: 0,
            cheerleaders: 0,
            assistant_coaches: 0,
            fan_factor: 0,
            dedicated_fans: 0,
            team_value: 0,
            treasury: 0,
            special_rules: vec![],
            players: vec![],
            vampire_lord: false,
            necromancer: false,
        };
        let away = Team { id: "t2".into(), ..home.clone() };
        let mut game = Game::new(home, away, ffb_model::enums::Rules::Bb2025);

        apply_player_markers(&mut game, true, vec![("ghost".to_string(), "Nice job".to_string())]);

        assert!(game.field_model.get_player_marker("ghost").is_none());
    }
}
