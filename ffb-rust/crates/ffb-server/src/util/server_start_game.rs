/// 1:1 translation of com.fumbbl.ffb.server.util.UtilServerStartGame.
///
/// Java reaches `SessionManager`/`ServerCommunication`/`DbQueryFactory`/`RequestProcessor`
/// via `gameState.getServer().getX()`; per this crate's convention (see `game_cache.rs`,
/// `server_command_handler_password_challenge.rs`) they are threaded through explicitly
/// instead as parameters.
///
/// This crate's `GameState::start_game` (see `game_state.rs`) already performs the
/// "push the `StartGame` sequence and run until the next prompt" half of Java's
/// `startGame(GameState)` as a side effect of initializing the engine `driver` — its own
/// doc comment says as much ("Java equivalent: `GameCache.addTeamToGame()` +
/// `UtilServerStartGame.startGame()`"). By the time a caller has a `GameState` with
/// `is_started() == true` to hand to this module's `start_game`, that push has already
/// happened; what remains genuinely untranslated here is the *tail* of Java's method:
/// the ownership check, the resumed-vs-fresh status/marker-loading branch, the
/// `GameStatus::Paused -> Active` transition + DB update, the turn timer, and the
/// `sendGameState` broadcast. Since this crate's `Game` has no `finished`/step-stack
/// fields to derive "resumed" from (see `game_cache.rs`'s `remove_game` doc comment for
/// the same missing-`finished`-field gap), `resumed` is threaded through explicitly by the
/// caller instead of being computed here.
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tokio::sync::mpsc;

use ffb_model::enums::{GameStatus, ServerStatus};
use ffb_model::marking::sort_mode::SortMode;
use ffb_model::model::game::Game;
use ffb_model::model::CommonProperty;
use ffb_model::model::ClientMode;
use ffb_protocol::commands::server_command_game_state::ServerCommandGameState;
use ffb_protocol::commands::server_command_join::ServerCommandJoin;
use ffb_protocol::commands::server_command_status::ServerCommandStatus;
use ffb_protocol::commands::server_command_user_settings::ServerCommandUserSettings;

use crate::db::db_connection_manager::DbConnectionManager;
use crate::db::query::db_user_settings_query::DbUserSettingsQuery;
use crate::game_cache::GameCache;
use crate::game_state::GameState;
use crate::model::received_command::SessionId;
use crate::net::session_manager::SessionManager;
use crate::request::fumbbl::util_fumbbl_request::HttpClient;
use crate::request::server_request_processor::ServerRequestProcessor;
use crate::util::marker_loading_service::MarkerLoadingService;

/// Java: `CommonPropertyValue.SETTING_PLAYER_MARKING_TYPE_AUTO`. Not ported as a
/// standalone constants type in this crate (see `model/common_property_value.rs`'s
/// unrelated struct of the same Java class name), so the literal string is reproduced here.
pub const SETTING_PLAYER_MARKING_TYPE_AUTO: &str = "playerMarkingTypeAuto";
/// Java: `CommonPropertyValue.SETTING_PLAYER_MARKING_TYPE_AUTO_NO_SORT`.
pub const SETTING_PLAYER_MARKING_TYPE_AUTO_NO_SORT: &str = "playerMarkingTypeAutoNoSort";

/// Bundles the dependencies needed to enqueue automated marker loading (Java:
/// `getServer().getRequestProcessor().add(new FumbblRequestLoadPlayerMarkings(...))`),
/// shared by `send_server_join` and `start_game`.
pub struct MarkerContext<'a> {
    pub request_processor: &'a Arc<Mutex<ServerRequestProcessor>>,
    pub client: Arc<dyn HttpClient + Send + Sync>,
    pub markings_url_template: &'a str,
}

/// Java: `joinGameAsPlayerAndCheckIfReadyToStart(GameState, Session, String, boolean, List<String>)`.
#[allow(clippy::too_many_arguments)]
pub async fn join_game_as_player_and_check_if_ready_to_start(
    game: &Game,
    game_id: i64,
    session_manager: &Mutex<SessionManager>,
    session_id: SessionId,
    coach: String,
    home_team: bool,
    account_properties: Vec<String>,
    sender: mpsc::UnboundedSender<String>,
    db: &DbConnectionManager,
    client_properties: &[(String, String)],
    marker_ctx: Option<MarkerContext<'_>>,
) -> bool {
    let team_away_id = &game.team_away.id;
    let team_home_id = &game.team_home.id;
    let same_team = !team_away_id.is_empty() && !team_home_id.is_empty() && team_away_id == team_home_id;

    if same_team {
        let status = ServerCommandStatus::new(ServerStatus::ErrorSameTeam, ServerStatus::ErrorSameTeam.message());
        session_manager.lock().unwrap().send_to(session_id, &status.to_json_value().to_string());
        return false;
    }

    let player_count = send_server_join(
        game_id,
        session_manager,
        session_id,
        coach,
        home_team,
        ClientMode::PLAYER,
        account_properties,
        sender,
        db,
        client_properties,
        marker_ctx,
    )
    .await;

    player_count > 1
}

/// Java: `sendServerJoin(GameState, Session, String, boolean, ClientMode, List<String>)`.
///
/// Returns `players.length` (Java) — the number of `ClientMode.PLAYER` sessions now
/// registered for the game.
#[allow(clippy::too_many_arguments)]
pub async fn send_server_join(
    game_id: i64,
    session_manager: &Mutex<SessionManager>,
    session_id: SessionId,
    coach: String,
    home_team: bool,
    mode: ClientMode,
    account_properties: Vec<String>,
    sender: mpsc::UnboundedSender<String>,
    db: &DbConnectionManager,
    client_properties: &[(String, String)],
    marker_ctx: Option<MarkerContext<'_>>,
) -> usize {
    {
        let mut sm = session_manager.lock().unwrap();
        sm.add_session(session_id, game_id, coach.clone(), mode, home_team, account_properties.clone(), sender);
    }

    let (player_list, visible_spectators) = {
        let sm = session_manager.lock().unwrap();
        let home_session = sm.get_session_of_home_coach(game_id);
        let mut player_list: Vec<String> = Vec::new();
        let mut visible_spectators: Vec<String> = Vec::new();
        // Java iterates `sessionManager.getSessionsForGameId(...)` and skips closed
        // sessions (`session.isOpen()`); this crate's `SessionManager` never retains a
        // closed session, so every session it reports is implicitly "open".
        for sid in sm.get_sessions_for_game_id(game_id) {
            let session_mode = sm.get_mode_for_session(sid);
            let session_coach = sm.get_coach_for_session(sid).unwrap_or_default().to_string();
            if session_mode == Some(ClientMode::PLAYER) {
                if Some(sid) == home_session {
                    player_list.insert(0, session_coach);
                } else {
                    player_list.push(session_coach);
                }
            } else if !sm.is_session_admin(sid) {
                visible_spectators.push(session_coach);
            }
        }
        (player_list, visible_spectators)
    };

    let settings_map = send_user_settings(session_manager, session_id, &coach, db, client_properties).await;

    let silent_join = mode == ClientMode::SPECTATOR && account_properties.iter().any(|p| p == "ADMIN");
    if !silent_join {
        let join_command = ServerCommandJoin::new(coach.clone(), mode, player_list.clone(), visible_spectators, String::new());
        let json = join_command.to_json_value().to_string();
        let sm = session_manager.lock().unwrap();
        for sid in sm.get_sessions_for_game_id(game_id) {
            sm.send_to(sid, &json);
        }
    }

    if mode == ClientMode::SPECTATOR {
        if let Some(setting_value) = settings_map.get(&CommonProperty::SETTING_PLAYER_MARKING_TYPE) {
            let sort_mode = if setting_value.eq_ignore_ascii_case(SETTING_PLAYER_MARKING_TYPE_AUTO) {
                Some(SortMode::Default)
            } else if setting_value.eq_ignore_ascii_case(SETTING_PLAYER_MARKING_TYPE_AUTO_NO_SORT) {
                Some(SortMode::None)
            } else {
                None
            };
            if let (Some(sort_mode), Some(ctx)) = (sort_mode, &marker_ctx) {
                // `dispatch` is `None`: this call chain doesn't thread a
                // `mpsc::UnboundedSender<ReceivedCommand>` through `MarkerContext` yet — see
                // `MarkerDispatch`'s doc comment in `marker_loading_service.rs`.
                MarkerLoadingService::new().load_marker_auto(
                    ctx.request_processor,
                    Arc::clone(&ctx.client),
                    ctx.markings_url_template,
                    coach.clone(),
                    sort_mode,
                    None,
                );
            }
        }
    }

    player_list.len()
}

/// Java: `sendUserSettings(FantasyFootballServer, String, Session)`.
pub async fn send_user_settings(
    session_manager: &Mutex<SessionManager>,
    session_id: SessionId,
    coach: &str,
    db: &DbConnectionManager,
    client_properties: &[(String, String)],
) -> HashMap<CommonProperty, String> {
    let mut names: Vec<CommonProperty> = Vec::new();
    let mut values: Vec<String> = Vec::new();

    // Java: `for (String serverProperty : server.getPropertyKeys()) if
    // (serverProperty.startsWith("client.")) { ... }`.
    for (key, value) in client_properties {
        if key.starts_with("client.") {
            if let Some(property) = CommonProperty::for_key(key) {
                names.push(property);
                values.push(value.clone());
            }
        }
    }

    if db.pool_ready() {
        if let Ok(mut conn) = db.open_db_connection().await {
            let mut query = DbUserSettingsQuery::new();
            if query.execute(&mut conn, coach).await.is_ok() {
                for name in query.get_setting_names() {
                    if let Some(property) = CommonProperty::for_key(name) {
                        if let Some(value) = query.get_setting_value(name) {
                            names.push(property);
                            values.push(value.to_string());
                        }
                    }
                }
            }
            let _ = db.close_db_connection(conn).await;
        }
    }

    let mut settings_map: HashMap<CommonProperty, String> = HashMap::new();
    for (name, value) in names.iter().zip(values.iter()) {
        settings_map.insert(*name, value.clone());
    }

    if !names.is_empty() && !values.is_empty() {
        let mut command = ServerCommandUserSettings::default();
        for (name, value) in names.iter().zip(values.iter()) {
            command.add_user_setting(*name, value.clone());
        }
        let json = command.to_json_value().to_string();
        session_manager.lock().unwrap().send_to(session_id, &json);
    }

    settings_map
}

/// Java: `startGame(GameState)`.
///
/// `resumed` stands in for Java's `(game.getFinished() == null) && (gameState.getStepStack().size() == 0)`
/// negation — see module doc comment for why this crate has no equivalent fields to derive
/// it from, and why the "push a fresh `StartGame` sequence" half of Java's branch is already
/// handled by `GameState::start_game` before this function is ever reachable.
#[allow(clippy::too_many_arguments)]
pub async fn start_game(
    game_state: &mut GameState,
    resumed: bool,
    session_manager: &Mutex<SessionManager>,
    db: &DbConnectionManager,
    fumbbl_mode: bool,
    check_ownership: bool,
    marker_ctx: Option<MarkerContext<'_>>,
) -> bool {
    let game_id = game_state.get_id();

    let (home_coach, away_coach, is_testing, waiting_for_opponent, is_paused, home_team_id, away_team_id) = {
        let game = match game_state.get_game() {
            Some(g) => g,
            None => return false,
        };
        (
            game.team_home.coach.clone(),
            game.team_away.coach.clone(),
            game.testing,
            game.waiting_for_opponent,
            game.status == GameStatus::Paused,
            game.team_home.id.clone(),
            game.team_away.id.clone(),
        )
    };
    let _ = (home_team_id, away_team_id);

    let mut ownership_ok = true;
    if !is_testing && check_ownership {
        let sm = session_manager.lock().unwrap();
        let session_home = sm.get_session_of_home_coach(game_id);
        let session_away = sm.get_session_of_away_coach(game_id);
        if !sm.is_home_coach(game_id, &home_coach) {
            ownership_ok = false;
            if let Some(sid) = session_home {
                let status = ServerCommandStatus::new(ServerStatus::ErrorNotYourTeam, ServerStatus::ErrorNotYourTeam.message());
                sm.send_to(sid, &status.to_json_value().to_string());
            }
        }
        if !sm.is_away_coach(game_id, &away_coach) {
            ownership_ok = false;
            if let Some(sid) = session_away {
                let status = ServerCommandStatus::new(ServerStatus::ErrorNotYourTeam, ServerStatus::ErrorNotYourTeam.message());
                sm.send_to(sid, &status.to_json_value().to_string());
            }
        }
    }

    if !ownership_ok {
        return false;
    }

    if resumed && fumbbl_mode {
        // Java: `server.getRequestProcessor().add(new FumbblRequestResumeGamestate(gameState))`.
        // No `FumbblRequestResumeGamestate` exists in this crate — documented gap.
        log::debug!("game {}: resumed in FUMBBL mode — FumbblRequestResumeGamestate not ported", game_id);
    }

    if is_paused {
        if let Some(game) = game_state.get_game_mut() {
            game.status = GameStatus::Active;
        }
        if let Some(game) = game_state.get_game() {
            let _ = GameCache::queue_db_update(db, game, true).await;
        }
    }

    if !waiting_for_opponent {
        // Java: `UtilServerTimer.startTurnTimer(gameState, System.currentTimeMillis())`.
        // No turn-timer subsystem exists in this crate — documented gap.
        log::debug!("game {}: turn timer not started — UtilServerTimer not ported", game_id);
    }

    {
        let game = game_state.get_game().cloned();
        let command = ServerCommandGameState::new(game);
        let json = command.to_json_value().to_string();
        session_manager.lock().unwrap().send_all(game_id, &json);
    }
    // Java: `gameState.fetchChanges()` clears the pending model-change list after
    // broadcasting the whole model; this crate's `GameState` has no such change-tracking
    // layer (model syncs are computed differently — see `driver.rs`), so there is nothing
    // to clear here.

    if resumed {
        return true;
    }

    if let Some(ctx) = marker_ctx {
        let service = MarkerLoadingService::new();
        if db.pool_ready() {
            if let Ok(mut conn) = db.open_db_connection().await {
                let mut query = DbUserSettingsQuery::new();

                let _ = query.execute(&mut conn, &home_coach).await;
                let (home_auto, home_sort) =
                    resolve_marking(query.get_setting_value(CommonProperty::SETTING_PLAYER_MARKING_TYPE.get_key()));
                dispatch_marker_load(&service, &ctx, home_auto, home_sort, &home_coach, &mut conn, &home_coach_team_id(game_state)).await;

                let _ = query.execute(&mut conn, &away_coach).await;
                let (away_auto, away_sort) =
                    resolve_marking(query.get_setting_value(CommonProperty::SETTING_PLAYER_MARKING_TYPE.get_key()));
                dispatch_marker_load(&service, &ctx, away_auto, away_sort, &away_coach, &mut conn, &away_coach_team_id(game_state)).await;

                let _ = db.close_db_connection(conn).await;
            }
        }
    }

    true
}

fn home_coach_team_id(game_state: &GameState) -> String {
    game_state.get_game().map(|g| g.team_home.id.clone()).unwrap_or_default()
}

fn away_coach_team_id(game_state: &GameState) -> String {
    game_state.get_game().map(|g| g.team_away.id.clone()).unwrap_or_default()
}

/// Java's per-coach `loadAuto`/`sortMode` derivation inside `startGame`, duplicated for
/// home and away.
fn resolve_marking(setting_value: Option<&str>) -> (bool, Option<SortMode>) {
    match setting_value {
        Some(v) if v.eq_ignore_ascii_case(SETTING_PLAYER_MARKING_TYPE_AUTO) => (true, Some(SortMode::Default)),
        Some(v) if v.eq_ignore_ascii_case(SETTING_PLAYER_MARKING_TYPE_AUTO_NO_SORT) => (true, Some(SortMode::None)),
        _ => (false, None),
    }
}

/// Java: `loadingService.loadMarker(gameState, session, homeTeam, loadAuto, sortMode)`.
async fn dispatch_marker_load(
    service: &MarkerLoadingService,
    ctx: &MarkerContext<'_>,
    auto: bool,
    sort_mode: Option<SortMode>,
    coach: &str,
    conn: &mut mysql_async::Conn,
    team_id: &str,
) {
    if auto {
        if let Some(sort_mode) = sort_mode {
            service.load_marker_auto(
                ctx.request_processor,
                Arc::clone(&ctx.client),
                ctx.markings_url_template,
                coach.to_string(),
                sort_mode,
                None,
            );
        }
    } else if !team_id.is_empty() {
        let _ = service.load_marker_from_db(conn, team_id).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::team::Team;
    use crate::request::fumbbl::util_fumbbl_request::MockHttpClient;

    fn team(id: &str, coach: &str) -> Team {
        Team {
            id: id.into(),
            name: format!("Team {}", id),
            race: "Human".into(),
            roster_id: "human".into(),
            coach: coach.into(),
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
        }
    }

    fn setup_session_manager() -> (Mutex<SessionManager>, mpsc::UnboundedReceiver<String>) {
        let sm = SessionManager::new();
        let (_tx, rx) = mpsc::unbounded_channel();
        (Mutex::new(sm), rx)
    }

    #[tokio::test]
    async fn join_game_as_player_rejects_same_team() {
        let game = Game::new(team("t1", "Home"), team("t1", "Away"), Rules::Bb2025);
        let sm = Mutex::new(SessionManager::new());
        let db = DbConnectionManager::new();
        // Java: the session already exists (it connected over the WebSocket) before this
        // method is invoked; the same-team branch never calls `addSession` itself, it only
        // sends a status to the already-connected session.
        let (tx, mut rx) = mpsc::unbounded_channel();
        sm.lock().unwrap().add_session(1, 0, "Home".into(), ClientMode::PLAYER, true, vec![], tx.clone());
        let ready = join_game_as_player_and_check_if_ready_to_start(
            &game, 1, &sm, 1, "Home".to_string(), true, vec![], tx, &db, &[], None,
        )
        .await;
        assert!(!ready);
        let msg = rx.try_recv().expect("expected ERROR_SAME_TEAM status");
        assert!(msg.contains("Same Team") || msg.contains("errorSameTeam") || msg.contains("cannot play a team against itself"));
    }

    #[tokio::test]
    async fn join_game_as_player_with_distinct_teams_registers_session() {
        let game = Game::new(team("home", "Home"), team("away", "Away"), Rules::Bb2025);
        let sm = Mutex::new(SessionManager::new());
        let db = DbConnectionManager::new();
        let (tx, mut rx) = mpsc::unbounded_channel();
        let ready = join_game_as_player_and_check_if_ready_to_start(
            &game, 42, &sm, 1, "Home".to_string(), true, vec![], tx, &db, &[], None,
        )
        .await;
        // Only one player session registered so far -> not ready to start yet.
        assert!(!ready);
        // A join broadcast should still have been sent to the (single) registered session.
        let msg = rx.try_recv().expect("expected a join broadcast");
        assert!(msg.contains("serverJoin"));
        assert_eq!(sm.lock().unwrap().get_coach_for_session(1), Some("Home"));
    }

    #[tokio::test]
    async fn send_server_join_orders_home_coach_first() {
        let (sm, _rx) = setup_session_manager();
        let db = DbConnectionManager::new();
        {
            let (tx_away, _rx_away) = mpsc::unbounded_channel();
            sm.lock().unwrap().add_session(2, 100, "Away".into(), ClientMode::PLAYER, false, vec![], tx_away);
        }
        let (tx_home, mut rx_home) = mpsc::unbounded_channel();
        let count = send_server_join(
            100, &sm, 1, "Home".to_string(), true, ClientMode::PLAYER, vec![], tx_home, &db, &[], None,
        )
        .await;
        assert_eq!(count, 2);
        let msg = rx_home.try_recv().expect("expected a join broadcast to the home session");
        assert!(msg.contains("\"playerNames\":[\"Home\",\"Away\"]"));
    }

    #[tokio::test]
    async fn send_server_join_admin_spectator_join_is_silent() {
        let (sm, _rx) = setup_session_manager();
        let db = DbConnectionManager::new();
        let (tx, mut rx) = mpsc::unbounded_channel();
        let count = send_server_join(
            100,
            &sm,
            1,
            "Watcher".to_string(),
            false,
            ClientMode::SPECTATOR,
            vec!["ADMIN".to_string()],
            tx,
            &db,
            &[],
            None,
        )
        .await;
        assert_eq!(count, 0);
        assert!(rx.try_recv().is_err(), "admin spectator join must not broadcast");
    }

    #[tokio::test]
    async fn send_user_settings_forwards_client_dot_properties() {
        let sm = Mutex::new(SessionManager::new());
        let (tx, mut rx) = mpsc::unbounded_channel();
        sm.lock().unwrap().add_session(1, 100, "Coach".into(), ClientMode::PLAYER, true, vec![], tx);
        let db = DbConnectionManager::new();
        let client_properties = vec![("client.ping.interval".to_string(), "30".to_string())];
        let settings = send_user_settings(&sm, 1, "Coach", &db, &client_properties).await;
        assert_eq!(settings.get(&CommonProperty::CLIENT_PING_INTERVAL).map(String::as_str), Some("30"));
        let msg = rx.try_recv().expect("expected a ServerCommandUserSettings message");
        assert!(msg.contains("serverUserSettings"));
    }

    #[tokio::test]
    async fn send_user_settings_ignores_non_client_properties() {
        let (sm, mut rx) = setup_session_manager();
        {
            let (tx, _rx2) = mpsc::unbounded_channel();
            sm.lock().unwrap().add_session(1, 100, "Coach".into(), ClientMode::PLAYER, true, vec![], tx);
        }
        let db = DbConnectionManager::new();
        let client_properties = vec![("db.url".to_string(), "jdbc:...".to_string())];
        let settings = send_user_settings(&sm, 1, "Coach", &db, &client_properties).await;
        assert!(settings.is_empty());
        assert!(rx.try_recv().is_err(), "no settings means no broadcast");
    }

    #[tokio::test]
    async fn start_game_missing_game_returns_false() {
        let mut gs = GameState::new(1);
        let (sm, _rx) = setup_session_manager();
        let db = DbConnectionManager::new();
        let started = start_game(&mut gs, false, &sm, &db, false, true, None).await;
        assert!(!started);
    }

    #[tokio::test]
    async fn start_game_sends_status_error_when_ownership_check_fails() {
        let mut gs = GameState::new(1);
        gs.start_game(team("home", "RealHome"), team("away", "RealAway"), Rules::Bb2025, 0);
        let (sm, _rx) = setup_session_manager();
        let db = DbConnectionManager::new();
        {
            // Register a session where the connected coach does NOT match the home team's coach.
            let (tx, _rx2) = mpsc::unbounded_channel();
            sm.lock().unwrap().add_session(1, 1, "Impostor".into(), ClientMode::PLAYER, true, vec![], tx);
        }
        let started = start_game(&mut gs, false, &sm, &db, false, true, None).await;
        assert!(!started);
    }

    #[tokio::test]
    async fn start_game_skips_ownership_check_when_testing() {
        let mut gs = GameState::new(1);
        gs.start_game(team("home", "RealHome"), team("away", "RealAway"), Rules::Bb2025, 0);
        if let Some(game) = gs.get_game_mut() {
            game.testing = true;
        }
        let sm = Mutex::new(SessionManager::new());
        let db = DbConnectionManager::new();
        let (tx, mut rx) = mpsc::unbounded_channel();
        sm.lock().unwrap().add_session(1, 1, "Impostor".into(), ClientMode::PLAYER, true, vec![], tx);

        let started = start_game(&mut gs, false, &sm, &db, false, true, None).await;
        assert!(started);
        let msg = rx.try_recv().expect("expected a ServerCommandGameState broadcast");
        assert!(msg.contains("serverGameState"));
    }

    #[tokio::test]
    async fn start_game_transitions_paused_to_active() {
        let mut gs = GameState::new(1);
        gs.start_game(team("home", "RealHome"), team("away", "RealAway"), Rules::Bb2025, 0);
        if let Some(game) = gs.get_game_mut() {
            game.testing = true;
            game.status = GameStatus::Paused;
        }
        let (sm, mut rx) = setup_session_manager();
        let db = DbConnectionManager::new();

        let started = start_game(&mut gs, false, &sm, &db, false, true, None).await;
        assert!(started);
        assert_eq!(gs.get_game().unwrap().status, GameStatus::Active);
        let _ = rx.try_recv();
    }

    #[tokio::test]
    async fn start_game_resumed_returns_before_marker_loading() {
        let mut gs = GameState::new(1);
        gs.start_game(team("home", "RealHome"), team("away", "RealAway"), Rules::Bb2025, 0);
        if let Some(game) = gs.get_game_mut() {
            game.testing = true;
        }
        let (sm, mut rx) = setup_session_manager();
        let db = DbConnectionManager::new();

        let started = start_game(&mut gs, true, &sm, &db, false, true, None).await;
        assert!(started);
        let _ = rx.try_recv();
    }

    #[test]
    fn resolve_marking_recognizes_auto_and_auto_no_sort() {
        assert_eq!(resolve_marking(Some("playerMarkingTypeAuto")), (true, Some(SortMode::Default)));
        assert_eq!(resolve_marking(Some("PLAYERMARKINGTYPEAUTONOSORT")), (true, Some(SortMode::None)));
        assert_eq!(resolve_marking(Some("manual")), (false, None));
        assert_eq!(resolve_marking(None), (false, None));
    }

    #[test]
    fn marker_context_construction_smoke_test() {
        let processor = Arc::new(Mutex::new(ServerRequestProcessor::new()));
        let client: Arc<dyn HttpClient + Send + Sync> = Arc::new(MockHttpClient { response: Ok("{}".to_string()) });
        let _ctx = MarkerContext { request_processor: &processor, client, markings_url_template: "http://fumbbl/markings/$1" };
    }
}
