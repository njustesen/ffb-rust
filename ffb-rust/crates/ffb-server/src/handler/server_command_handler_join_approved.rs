/// 1:1 translation of com.fumbbl.ffb.server.handler.ServerCommandHandlerJoinApproved.
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use ffb_model::enums::{GameStatus, NetCommandId, ServerStatus};
use ffb_model::model::ClientMode;
use ffb_model::model::team_list::TeamList;
use ffb_model::model::team_list_entry::TeamListEntry;
use ffb_protocol::commands::server_command_game_state::ServerCommandGameState;
use ffb_protocol::commands::server_command_status::ServerCommandStatus;
use ffb_protocol::commands::server_command_team_list::ServerCommandTeamList;
use crate::db::db_connection_manager::DbConnectionManager;
use crate::game_cache::GameCache;
use crate::model::received_command::SessionId;
use crate::net::commands::internal_server_command::InternalServerCommand;
use crate::net::commands::internal_server_command_join_approved::InternalServerCommandJoinApproved;
use crate::net::session_manager::SessionManager;
use crate::roster_cache::RosterCache;
use crate::team_cache::TeamCache;
use crate::util::server_start_game::{
    join_game_as_player_and_check_if_ready_to_start, send_server_join, send_user_settings, start_game,
};

/// Java: `ServerCommandHandlerJoinApproved.TEST_PREFIX`.
const TEST_PREFIX: &str = "test:";

/// Java: `ServerCommandHandlerJoinApproved extends ServerCommandHandler`.
pub struct ServerCommandHandlerJoinApproved {
    game_cache: Arc<Mutex<GameCache>>,
    session_manager: Arc<Mutex<SessionManager>>,
    db_connection_manager: Arc<Mutex<DbConnectionManager>>,
    team_cache: Arc<TeamCache>,
    roster_cache: Arc<RosterCache>,
    /// Java: `for (String serverProperty : server.getPropertyKeys())` — the `client.*`
    /// server properties `UtilServerStartGame.sendUserSettings` forwards. Threaded through
    /// explicitly (this crate's convention — see `util/server_start_game.rs`'s module doc
    /// comment) rather than reached via a `getServer()` singleton.
    client_properties: Arc<Vec<(String, String)>>,
}

impl ServerCommandHandlerJoinApproved {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        game_cache: Arc<Mutex<GameCache>>,
        session_manager: Arc<Mutex<SessionManager>>,
        db_connection_manager: Arc<Mutex<DbConnectionManager>>,
        team_cache: Arc<TeamCache>,
        roster_cache: Arc<RosterCache>,
        client_properties: Arc<Vec<(String, String)>>,
    ) -> Self {
        Self { game_cache, session_manager, db_connection_manager, team_cache, roster_cache, client_properties }
    }

    /// Java: `getId()` — returns `NetCommandId.INTERNAL_SERVER_JOIN_APPROVED`.
    pub fn get_id(&self) -> NetCommandId {
        NetCommandId::InternalServerJoinApproved
    }

    /// Java: `loadGameStateById(InternalServerCommandJoinApproved)`.
    ///
    /// Java falls back to `GameCache.queryFromDb(gameId)` when the game isn't
    /// cached in memory; the Rust `GameCache` has no persistence layer, so a
    /// cache miss here simply reports "not found" (there is nothing further
    /// to query).
    fn load_game_state_by_id(&self, game_id: i64) -> bool {
        self.game_cache.lock().unwrap().get_game_state_by_id(game_id).is_some()
    }

    /// Java: `handleCommand(ReceivedCommand)`.
    ///
    /// `sender` supplies the outgoing channel for `SessionManager.add_session`
    /// (Java re-uses the already-connected Jetty `Session`; the Rust
    /// `SessionManager` needs the sender explicitly since there's no `Session`
    /// object to query).
    pub async fn handle_command(
        &self,
        join_approved_command: &InternalServerCommandJoinApproved,
        session_id: SessionId,
        sender: mpsc::UnboundedSender<String>,
    ) -> bool {
        let mut game_id = 0i64;
        let mut game_found = false;

        if join_approved_command.get_game_id() > 0 {
            game_id = join_approved_command.get_game_id();
            game_found = self.load_game_state_by_id(game_id);
        } else if !join_approved_command.get_game_name().is_empty() {
            let game_name = join_approved_command.get_game_name();
            let existing_id = {
                let gc = self.game_cache.lock().unwrap();
                gc.get_game_state_by_name(game_name).map(|gs| gs.get_id())
            };
            match existing_id {
                Some(id) => {
                    game_id = id;
                    game_found = true;
                }
                None => {
                    // Java: `!getServer().isBlockingNewGames()` gate — no equivalent
                    // server-wide flag exists yet, so new games are always created,
                    // matching the common (non-blocking) case.
                    let _testing = game_name.starts_with(TEST_PREFIX);
                    let mut gc = self.game_cache.lock().unwrap();
                    let id = gc.create_game_state();
                    gc.map_game_name_to_id(game_name.to_string(), id);
                    game_id = id;
                    game_found = true;
                }
            }
        }

        if game_found {
            let client_mode = parse_client_mode(join_approved_command.get_client_mode());
            if client_mode == Some(ClientMode::PLAYER) {
                self.handle_player_join(join_approved_command, session_id, sender, game_id).await;
            } else {
                self.handle_spectator_join(join_approved_command, session_id, sender, game_id).await;
            }
        } else if parse_client_mode(join_approved_command.get_client_mode()) == Some(ClientMode::REPLAY) {
            // Java: `UtilServerStartGame.sendUserSettings(...)`.
            let db = self.db_connection_manager.lock().unwrap().clone();
            send_user_settings(
                &self.session_manager,
                session_id,
                join_approved_command.get_coach(),
                &db,
                &self.client_properties,
            )
            .await;
        }

        true
    }

    /// Java: the `ClientMode.SPECTATOR` half of `handleCommand`.
    ///
    /// ```java
    /// closeOtherSessionWithThisCoach(gameState, joinApprovedCommand.getCoach(), session);
    /// sessionManager.addSession(session, gameState.getId(), joinApprovedCommand.getCoach(),
    ///     joinApprovedCommand.getClientMode(), false, joinApprovedCommand.getAccountProperties());
    /// UtilServerStartGame.sendServerJoin(gameState, session, joinApprovedCommand.getCoach(), false,
    ///     ClientMode.SPECTATOR, joinApprovedCommand.getAccountProperties());
    /// if (gameState.getGame().getStarted() != null) {
    ///     UtilServerTimer.syncTime(gameState, System.currentTimeMillis());
    ///     communication.sendGameState(session, gameState);
    /// }
    /// ```
    ///
    /// `send_server_join` (Phase ZX.3) already calls `SessionManager::add_session`
    /// internally (see `server_start_game.rs:125`), so the separate manual `add_session`
    /// call Java appears to make first is *not* duplicated here — a single call site
    /// matches Java's net effect (Java's own `sessionManager.addSession` is likewise not
    /// called a second time inside `sendServerJoin`), even though the call moves.
    async fn handle_spectator_join(
        &self,
        join_approved_command: &InternalServerCommandJoinApproved,
        session_id: SessionId,
        sender: mpsc::UnboundedSender<String>,
        game_id: i64,
    ) {
        let coach = join_approved_command.get_coach().to_string();
        {
            let mut sm = self.session_manager.lock().unwrap();
            if let Some(other) = sm.find_other_session_for_coach(game_id, &coach, session_id) {
                sm.remove_session(other);
            }
        }

        let db = self.db_connection_manager.lock().unwrap().clone();
        let _ = send_server_join(
            game_id,
            &self.session_manager,
            session_id,
            coach,
            false,
            ClientMode::SPECTATOR,
            join_approved_command.get_account_properties().to_vec(),
            sender,
            &db,
            &self.client_properties,
            None,
        )
        .await;

        // Java: `gameState.getGame().getStarted() != null` — this crate's `Game` has no
        // `started` date field (see `util/server_start_game.rs`'s module doc comment for
        // the same missing-field gap elsewhere in this phase); `GameState::is_started()`
        // (the engine driver being initialized) is the closest available proxy.
        let started = {
            let gc = self.game_cache.lock().unwrap();
            gc.get_game_state_by_id(game_id).map(|gs| gs.is_started()).unwrap_or(false)
        };
        if started {
            // Java: `UtilServerTimer.syncTime(gameState, System.currentTimeMillis())` — no
            // turn-timer subsystem exists in this crate to sync (same documented gap as
            // `util::server_start_game::start_game`).
            let game = {
                let gc = self.game_cache.lock().unwrap();
                gc.get_game_state_by_id(game_id).and_then(|gs| gs.get_game().cloned())
            };
            let command = ServerCommandGameState::new(game);
            let json = command.to_json_value().to_string();
            self.session_manager.lock().unwrap().send_to(session_id, &json);
        }
    }

    /// Java: the `ClientMode.PLAYER` half of `handleCommand`, dispatching to
    /// `joinWithoutTeam` / `joinWithTeam` / `sendTeamList`.
    async fn handle_player_join(
        &self,
        join_approved_command: &InternalServerCommandJoinApproved,
        session_id: SessionId,
        sender: mpsc::UnboundedSender<String>,
        game_id: i64,
    ) {
        let coach = join_approved_command.get_coach().to_string();
        let requested_team_id = join_approved_command.get_team_id().to_string();

        // Java: `Game game = gameState.getGame();` — Java's `GameState` always has a `Game`
        // (with possibly-blank team slots) once created; this crate's `GameState` has no
        // `Game` at all until both teams are attached via `start_game` (no empty/skeleton
        // `Game` slot exists here — see `server_command_handler_schedule_game.rs`'s doc
        // comment for the same structural gap). A freshly-created-by-name game (no
        // pre-scheduled teams) therefore has no `Game` to inspect yet; its team coach
        // fields are modeled as blank here, exactly matching what Java's own blank `Game`
        // would report — `coach.equalsIgnoreCase("")` never matches, so `is_home_coach`/
        // `is_away_coach` fall through below precisely as Java's would.
        let (home_coach, away_coach, home_team_id, testing, is_scheduled, started) = {
            let gc = self.game_cache.lock().unwrap();
            let gs = gc.get_game_state_by_id(game_id);
            let started = gs.map(|g| g.is_started()).unwrap_or(false);
            match gs.and_then(|g| g.get_game()) {
                Some(game) => (
                    game.team_home.coach.clone(),
                    game.team_away.coach.clone(),
                    game.team_home.id.clone(),
                    game.testing,
                    game.status == GameStatus::Scheduled,
                    started,
                ),
                None => (String::new(), String::new(), String::new(), false, false, started),
            }
        };

        let is_home_coach = !home_coach.is_empty() && coach.eq_ignore_ascii_case(&home_coach);
        let is_away_coach = !away_coach.is_empty() && coach.eq_ignore_ascii_case(&away_coach);

        if is_home_coach || is_away_coach {
            if is_scheduled || started {
                let home_team = is_home_coach;
                self.join_without_team(game_id, &coach, testing, home_team, join_approved_command, session_id, sender).await;
            } else if !requested_team_id.is_empty() {
                self.join_with_team(game_id, &coach, testing, &requested_team_id, &home_team_id, join_approved_command, session_id, sender).await;
            } else {
                self.send_team_list(&coach, &sender);
            }
        } else if started {
            // Java: `communication.sendStatus(session, ServerStatus.ERROR_GAME_IN_USE, null)`
            // — sent directly to the joining session's live channel, since (unlike the
            // SPECTATOR branch) no `add_session` has happened yet to register it with
            // `SessionManager`.
            let status_cmd = ServerCommandStatus::new(ServerStatus::ErrorGameInUse, ServerStatus::ErrorGameInUse.message());
            let _ = sender.send(status_cmd.to_json_value().to_string());
        } else if requested_team_id.is_empty() {
            self.send_team_list(&coach, &sender);
        } else {
            self.join_with_team(game_id, &coach, testing, &requested_team_id, &home_team_id, join_approved_command, session_id, sender).await;
        }
    }

    /// Java: `private void closeOtherSessionWithThisCoach(GameState, String, Session)`.
    ///
    /// Ported against `SessionManager` directly (the same substitution the SPECTATOR
    /// branch already made for `Java`'s `getServer().getCommunication().close(...)`),
    /// rather than threading a `ServerCommunication` reference through this handler too.
    fn close_other_session_with_this_coach(&self, game_id: i64, coach: &str, session_id: SessionId) {
        let mut sm = self.session_manager.lock().unwrap();
        if let Some(other) = sm.find_other_session_for_coach(game_id, coach, session_id) {
            sm.remove_session(other);
        }
    }

    /// Java: `private void joinWithoutTeam(GameState, InternalServerCommandJoinApproved, Session)`.
    async fn join_without_team(
        &self,
        game_id: i64,
        coach: &str,
        testing: bool,
        home_team: bool,
        join_approved_command: &InternalServerCommandJoinApproved,
        session_id: SessionId,
        sender: mpsc::UnboundedSender<String>,
    ) {
        if !testing {
            self.close_other_session_with_this_coach(game_id, coach, session_id);
        }

        let game = {
            let gc = self.game_cache.lock().unwrap();
            gc.get_game_state_by_id(game_id).and_then(|gs| gs.get_game().cloned())
        };
        let Some(game) = game else { return };

        let db = self.db_connection_manager.lock().unwrap().clone();
        let ready = join_game_as_player_and_check_if_ready_to_start(
            &game,
            game_id,
            &self.session_manager,
            session_id,
            coach.to_string(),
            home_team,
            join_approved_command.get_account_properties().to_vec(),
            sender,
            &db,
            &self.client_properties,
            None,
        )
        .await;

        if ready {
            // Java: FUMBBL-mode kickstart / `FumbblRequestCheckGamestate` dispatch — no
            // `ServerMode`/request-processor plumbing threaded into this handler (same
            // documented gap as `ServerCommandHandlerJoin`'s FUMBBL branch); only the
            // standalone tail is translated:
            // Java: `UtilServerStartGame.addDefaultGameOptions(gameState)` /
            // `UtilSkillBehaviours.registerBehaviours(...)` — no `GameOptions`/
            // skill-behaviour-registry subsystem exists in this crate to call into
            // (documented gap).
            self.run_start_game(game_id, &db).await;
        }
    }

    /// Java: `private void joinWithTeam(GameState, InternalServerCommandJoinApproved, Session)`.
    ///
    /// In this port, `handle_player_join`'s routing only reaches this method when no `Game`
    /// exists yet for `game_id` (a freshly-created-by-name game — see that method's doc
    /// comment on the structural gap this crate's driver-backed `GameState` has vs. Java's
    /// always-present blank `Game`). Team attachment below is therefore a documented no-op
    /// in that case (there is no `&mut Game` to attach into yet); it is still translated
    /// faithfully for whenever a `Game` *does* already exist (exercised directly by this
    /// file's unit tests), matching the actual Java method body 1:1.
    #[allow(clippy::too_many_arguments)]
    async fn join_with_team(
        &self,
        game_id: i64,
        coach: &str,
        testing: bool,
        team_id: &str,
        home_team_id: &str,
        join_approved_command: &InternalServerCommandJoinApproved,
        session_id: SessionId,
        sender: mpsc::UnboundedSender<String>,
    ) {
        if !testing {
            self.close_other_session_with_this_coach(game_id, coach, session_id);
        }

        // Java: `boolean homeTeam = (!StringTool.isProvided(game.getTeamHome().getId())
        //     || teamId.equals(game.getTeamHome().getId()));`
        let home_team = home_team_id.is_empty() || team_id == home_team_id;

        // Java: `Team teamSkeleton = getServer().getGameCache().getTeamSkeleton(teamId);
        //     getServer().getGameCache().addTeamToGame(pGameState, teamSkeleton, homeTeam);`
        // — this crate's `TeamSkeleton` is a distinct lightweight struct, not a `Team`
        // subtype (see `team_cache.rs`'s module doc comment), so it can't be handed to
        // `GameCache::add_team_to_game` (which requires a full `Team`). This placeholder
        // pre-registration step is skipped; the full team resolution Java performs moments
        // later in this same branch (below) is what actually matters for game state.
        let resolved_team = GameCache::get_team_by_id(team_id, &self.team_cache, &self.roster_cache);
        let Ok(resolved_team) = resolved_team else {
            log::error!("game {}: unable to resolve team {} for JoinApproved(PLAYER)", game_id, team_id);
            return;
        };

        {
            let mut gc = self.game_cache.lock().unwrap();
            if let Some(game) = gc.get_game_state_by_id_mut(game_id).and_then(|gs| gs.get_game_mut()) {
                GameCache::add_team_to_game(game, resolved_team, home_team);
            }
        }

        let game = {
            let gc = self.game_cache.lock().unwrap();
            gc.get_game_state_by_id(game_id).and_then(|gs| gs.get_game().cloned())
        };
        let Some(game) = game else { return };

        let db = self.db_connection_manager.lock().unwrap().clone();
        let ready = join_game_as_player_and_check_if_ready_to_start(
            &game,
            game_id,
            &self.session_manager,
            session_id,
            coach.to_string(),
            home_team,
            join_approved_command.get_account_properties().to_vec(),
            sender,
            &db,
            &self.client_properties,
            None,
        )
        .await;

        if ready {
            // Java: `UtilServerStartGame.addDefaultGameOptions(gameState)` /
            // `pGameState.initRulesDependentMembers()` / `pGameState.getGame().initializeRules()` /
            // `UtilSkillBehaviours.registerBehaviours(...)` — no `GameOptions`/rules-
            // reinitialization/skill-behaviour-registry subsystems exist in this crate to call
            // into (documented gap, matching this file's other `// java:` notes). What *is*
            // ported: re-resolving both teams by id and re-attaching them, exactly as Java's
            // `getTeamById`/`addTeamToGame` pair does immediately before `startGame`.
            let (home_id, away_id) = {
                let gc = self.game_cache.lock().unwrap();
                gc.get_game_state_by_id(game_id)
                    .and_then(|gs| gs.get_game())
                    .map(|g| (g.team_home.id.clone(), g.team_away.id.clone()))
                    .unwrap_or_default()
            };
            let resolved_home = GameCache::get_team_by_id(&home_id, &self.team_cache, &self.roster_cache);
            let resolved_away = GameCache::get_team_by_id(&away_id, &self.team_cache, &self.roster_cache);
            if let (Ok(resolved_home), Ok(resolved_away)) = (resolved_home, resolved_away) {
                let mut gc = self.game_cache.lock().unwrap();
                if let Some(game) = gc.get_game_state_by_id_mut(game_id).and_then(|gs| gs.get_game_mut()) {
                    GameCache::add_team_to_game(game, resolved_home, true);
                    GameCache::add_team_to_game(game, resolved_away, false);
                }
            }

            self.run_start_game(game_id, &db).await;
        }
    }

    /// Java: `UtilServerStartGame.startGame(gameState)`.
    ///
    /// Takes the `GameState` out of `GameCache` for the duration of the awaited call (see
    /// `GameCache::take_game_state`'s doc comment) so the `Mutex` guard isn't held across
    /// `start_game`'s internal `.await` points — required for
    /// `net::server_communication::dispatch_loop`'s `tokio::spawn`ed future to stay `Send`.
    async fn run_start_game(&self, game_id: i64, db: &DbConnectionManager) {
        let mut taken = { self.game_cache.lock().unwrap().take_game_state(game_id) };
        if let Some(gs) = taken.as_mut() {
            let _ = start_game(gs, false, &self.session_manager, db, false, true, None).await;
        }
        if let Some(gs) = taken {
            self.game_cache.lock().unwrap().add_game(gs);
        }
    }

    /// Java: `private void sendTeamList(GameState, InternalServerCommandJoinApproved, Session)`.
    ///
    /// Java: `pGameState.initRulesDependentMembers(); pGameState.getGame().initializeRules();`
    /// — no rules-(re)initialization subsystem exists on this crate's `GameState`/`Game` to
    /// call into (a `Game`'s rules are fixed at `start_game` construction time), so there is
    /// nothing to invoke here — documented gap, not invented logic.
    ///
    /// Sent directly to `sender` (the joining session's live outgoing channel) rather than
    /// through `SessionManager::send_to`, since — unlike the SPECTATOR branch — no
    /// `add_session` call has registered this session yet at this point in the flow.
    fn send_team_list(&self, coach: &str, sender: &mpsc::UnboundedSender<String>) {
        let teams = GameCache::get_teams_for_coach(coach, &self.team_cache, &self.roster_cache).unwrap_or_default();
        let mut team_list = TeamList::new();
        team_list.coach = Some(coach.to_string());
        for team in teams {
            team_list.add(TeamListEntry {
                team_id: team.id,
                team_name: team.name,
                coach: team.coach,
                race: team.race,
            });
        }
        let command = ServerCommandTeamList::new(team_list);
        let json = command.to_json_value().to_string();
        let _ = sender.send(json);
    }
}

/// Java's `ClientMode` enum constant name (`"PLAYER"`, `"SPECTATOR"`,
/// `"REPLAY"`), stored as a plain `String` on
/// `InternalServerCommandJoinApproved` pending full enum wiring there.
fn parse_client_mode(raw: &str) -> Option<ClientMode> {
    match raw.to_ascii_uppercase().as_str() {
        "PLAYER" => Some(ClientMode::PLAYER),
        "SPECTATOR" => Some(ClientMode::SPECTATOR),
        "REPLAY" => Some(ClientMode::REPLAY),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::team::Team;

    fn setup() -> (
        Arc<Mutex<GameCache>>,
        Arc<Mutex<SessionManager>>,
        Arc<Mutex<DbConnectionManager>>,
        Arc<TeamCache>,
        Arc<RosterCache>,
        Arc<Vec<(String, String)>>,
    ) {
        (
            Arc::new(Mutex::new(GameCache::new())),
            Arc::new(Mutex::new(SessionManager::new())),
            Arc::new(Mutex::new(DbConnectionManager::new())),
            Arc::new(TeamCache::new()),
            Arc::new(RosterCache::new()),
            Arc::new(Vec::new()),
        )
    }

    fn handler(
        gc: Arc<Mutex<GameCache>>,
        sm: Arc<Mutex<SessionManager>>,
        db: Arc<Mutex<DbConnectionManager>>,
        tc: Arc<TeamCache>,
        rc: Arc<RosterCache>,
        cp: Arc<Vec<(String, String)>>,
    ) -> ServerCommandHandlerJoinApproved {
        ServerCommandHandlerJoinApproved::new(gc, sm, db, tc, rc, cp)
    }

    fn team(id: &str, coach: &str) -> Team {
        Team {
            id: id.into(), name: format!("Team {}", id), race: "Human".into(), roster_id: "human".into(),
            coach: coach.into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![], vampire_lord: false, necromancer: false,
        }
    }

    #[test]
    fn construct() {
        let (gc, sm, db, tc, rc, cp) = setup();
        let _ = handler(gc, sm, db, tc, rc, cp);
    }

    #[tokio::test]
    async fn get_id_is_internal_server_join_approved() {
        let (gc, sm, db, tc, rc, cp) = setup();
        let h = handler(gc, sm, db, tc, rc, cp);
        assert_eq!(h.get_id(), NetCommandId::InternalServerJoinApproved);
    }

    #[tokio::test]
    async fn spectator_join_creates_game_by_name_and_broadcasts_real_join() {
        let (gc, sm, db, tc, rc, cp) = setup();
        let h = handler(Arc::clone(&gc), Arc::clone(&sm), db, tc, rc, cp);
        let cmd = InternalServerCommandJoinApproved::new(
            0, "NewGame".to_string(), "Watcher".to_string(), String::new(), "SPECTATOR".to_string(), vec![],
        );
        let (tx, mut rx) = mpsc::unbounded_channel();
        assert!(h.handle_command(&cmd, 7, tx).await);

        // The game should have been created and mapped by name...
        assert!(gc.lock().unwrap().get_game_state_by_name("NewGame").is_some());
        // ...and the spectator session registered by `send_server_join` itself.
        {
            let sm = sm.lock().unwrap();
            assert_eq!(sm.get_coach_for_session(7), Some("Watcher"));
            assert_eq!(sm.get_mode_for_session(7), Some(ClientMode::SPECTATOR));
        }
        let msg = rx.try_recv().expect("expected a real serverJoin broadcast");
        assert!(msg.contains("serverJoin"));
    }

    #[tokio::test]
    async fn spectator_join_closes_existing_session_for_same_coach() {
        let (gc, sm, db, tc, rc, cp) = setup();
        let game_id = gc.lock().unwrap().create_game_state();
        gc.lock().unwrap().map_game_name_to_id("ExistingGame".to_string(), game_id);
        {
            let (tx, _rx) = mpsc::unbounded_channel();
            sm.lock().unwrap().add_session(1, game_id, "Watcher".into(), ClientMode::SPECTATOR, false, vec![], tx);
        }
        let h = handler(Arc::clone(&gc), Arc::clone(&sm), db, tc, rc, cp);
        let cmd = InternalServerCommandJoinApproved::new(
            0, "ExistingGame".to_string(), "Watcher".to_string(), String::new(), "SPECTATOR".to_string(), vec![],
        );
        let (tx2, _rx2) = mpsc::unbounded_channel();
        assert!(h.handle_command(&cmd, 2, tx2).await);
        let sm = sm.lock().unwrap();
        assert_eq!(sm.get_game_id_for_session(1), 0, "old session should have been closed");
        assert_eq!(sm.get_coach_for_session(2), Some("Watcher"));
    }

    #[tokio::test]
    async fn spectator_join_of_already_started_game_sends_game_state() {
        let (gc, sm, db, tc, rc, cp) = setup();
        let game_id = gc.lock().unwrap().create_game_state();
        gc.lock().unwrap().get_game_state_by_id_mut(game_id).unwrap().start_game(
            team("home", "Alice"), team("away", "Bob"), Rules::Bb2025, 0,
        );
        let h = handler(Arc::clone(&gc), Arc::clone(&sm), db, tc, rc, cp);
        let cmd = InternalServerCommandJoinApproved::new(
            game_id, String::new(), "Watcher".to_string(), String::new(), "SPECTATOR".to_string(), vec![],
        );
        let (tx, mut rx) = mpsc::unbounded_channel();
        assert!(h.handle_command(&cmd, 9, tx).await);

        let mut saw_game_state = false;
        while let Ok(msg) = rx.try_recv() {
            if msg.contains("serverGameState") {
                saw_game_state = true;
            }
        }
        assert!(saw_game_state, "expected a serverGameState broadcast for an already-started game");
    }

    #[tokio::test]
    async fn player_join_without_team_when_already_scheduled_registers_session() {
        let (gc, sm, db, tc, rc, cp) = setup();
        let game_id = gc.lock().unwrap().create_game_state();
        gc.lock().unwrap().get_game_state_by_id_mut(game_id).unwrap().start_game(
            team("home", "Alice"), team("away", "Bob"), Rules::Bb2025, 0,
        );
        if let Some(game) = gc.lock().unwrap().get_game_state_by_id_mut(game_id).unwrap().get_game_mut() {
            game.testing = true;
        }
        let h = handler(Arc::clone(&gc), Arc::clone(&sm), db, tc, rc, cp);
        let cmd = InternalServerCommandJoinApproved::new(
            game_id, String::new(), "Alice".to_string(), String::new(), "PLAYER".to_string(), vec![],
        );
        let (tx, mut rx) = mpsc::unbounded_channel();
        assert!(h.handle_command(&cmd, 3, tx).await);

        assert_eq!(sm.lock().unwrap().get_coach_for_session(3), Some("Alice"));
        let msg = rx.try_recv().expect("expected a serverJoin broadcast");
        assert!(msg.contains("serverJoin"));
    }

    /// `join_with_team` is only reachable via `handle_player_join`'s routing when no `Game`
    /// exists yet for the game id (see that method's + `join_with_team`'s own doc comments
    /// for the structural reason), and that case can't itself attach a team (no `&mut Game`
    /// to mutate). So team resolution + attachment is exercised directly against a
    /// `GameState` that already has a `Game` (as if pre-scheduled), calling the private
    /// method directly — still real coverage of `TeamCache`/`RosterCache` resolution and
    /// `GameCache::add_team_to_game`, just not reached through the public dispatch path in
    /// this port.
    #[tokio::test]
    async fn join_with_team_resolves_via_team_and_roster_cache() {
        use std::fs;
        let team_dir = std::env::temp_dir().join("ffb_join_approved_test_teams");
        let roster_dir = std::env::temp_dir().join("ffb_join_approved_test_rosters");
        let _ = fs::remove_dir_all(&team_dir);
        let _ = fs::remove_dir_all(&roster_dir);
        fs::create_dir_all(&team_dir).unwrap();
        fs::create_dir_all(&roster_dir).unwrap();
        fs::write(
            team_dir.join("home.xml"),
            r#"<team id="home1"><name>Home Team</name><rosterId>human</rosterId><coach>Alice</coach><reRolls>1</reRolls></team>"#,
        ).unwrap();
        fs::write(
            team_dir.join("away.xml"),
            r#"<team id="away1"><name>Away Team</name><rosterId>human</rosterId><coach>Bob</coach></team>"#,
        ).unwrap();
        fs::write(
            roster_dir.join("human.xml"),
            r#"<roster id="human"><name>Human</name><reRollCost>50000</reRollCost></roster>"#,
        ).unwrap();

        let mut tc = TeamCache::new();
        tc.init(&team_dir).unwrap();
        let mut rc = RosterCache::new();
        rc.init(&roster_dir).unwrap();

        let (gc, sm, db, _tc, _rc, cp) = setup();
        let game_id = gc.lock().unwrap().create_game_state();
        gc.lock().unwrap().get_game_state_by_id_mut(game_id).unwrap().start_game(
            team("home1", "Alice"), team("away1", "Bob"), Rules::Bb2025, 0,
        );

        let h = handler(Arc::clone(&gc), Arc::clone(&sm), db, Arc::new(tc), Arc::new(rc), cp);
        let cmd = InternalServerCommandJoinApproved::new(
            game_id, String::new(), "Alice".to_string(), "home1".to_string(), "PLAYER".to_string(), vec![],
        );
        let (tx, _rx) = mpsc::unbounded_channel();
        h.join_with_team(game_id, "Alice", false, "home1", "home1", &cmd, 5, tx).await;

        let gc = gc.lock().unwrap();
        let game = gc.get_game_state_by_id(game_id).unwrap().get_game().unwrap();
        assert_eq!(game.team_home.name, "Home Team");
        assert_eq!(game.team_home.team_value, 50_000);

        let _ = fs::remove_dir_all(&team_dir);
        let _ = fs::remove_dir_all(&roster_dir);
    }

    /// A `PLAYER` join for a freshly-created (by-name) game — no `Game` exists yet, so
    /// `handle_player_join` treats both team-coach slots as blank (see that method's doc
    /// comment); with no team id supplied, this falls through to `sendTeamList`, matching
    /// Java's real "ad-hoc game, first player picks a team" flow.
    #[tokio::test]
    async fn player_join_no_team_id_on_fresh_game_sends_team_list() {
        let (gc, sm, db, tc, rc, cp) = setup();
        let game_id = gc.lock().unwrap().create_game_state();
        let h = handler(Arc::clone(&gc), Arc::clone(&sm), db, tc, rc, cp);
        let cmd = InternalServerCommandJoinApproved::new(
            game_id, String::new(), "Alice".to_string(), String::new(), "PLAYER".to_string(), vec![],
        );
        let (tx, mut rx) = mpsc::unbounded_channel();
        assert!(h.handle_command(&cmd, 6, tx).await);
        let msg = rx.try_recv().expect("expected a serverTeamList message");
        assert!(msg.contains("serverTeamList"));
    }

    /// A `PLAYER` join for an already-existing (started) game by a coach that matches
    /// neither team slot gets `ERROR_GAME_IN_USE` — `GameState::is_started()` (the engine
    /// driver being initialized) stands in for Java's `game.getStarted() != null` (see
    /// `handle_spectator_join`'s doc comment for the same missing-field substitution).
    #[tokio::test]
    async fn player_join_wrong_coach_with_started_game_gets_game_in_use_status() {
        let (gc, sm, db, tc, rc, cp) = setup();
        let game_id = gc.lock().unwrap().create_game_state();
        gc.lock().unwrap().get_game_state_by_id_mut(game_id).unwrap().start_game(
            team("home", "Alice"), team("away", "Bob"), Rules::Bb2025, 0,
        );
        let h = handler(Arc::clone(&gc), Arc::clone(&sm), db, tc, rc, cp);
        let cmd = InternalServerCommandJoinApproved::new(
            game_id, String::new(), "Stranger".to_string(), String::new(), "PLAYER".to_string(), vec![],
        );
        let (tx, mut rx) = mpsc::unbounded_channel();
        assert!(h.handle_command(&cmd, 8, tx).await);
        let msg = rx.try_recv().expect("expected a serverStatus message");
        assert!(msg.contains("Game In Use"));
    }

    #[tokio::test]
    async fn replay_mode_with_no_matching_game_sends_user_settings() {
        let (gc, sm, db, tc, rc, _cp) = setup();
        // No DB pool configured — `send_user_settings` still forwards `client.*` properties.
        let (tx, mut rx) = mpsc::unbounded_channel();
        sm.lock().unwrap().add_session(4, 0, "Coach".into(), ClientMode::PLAYER, true, vec![], tx);
        let cp = Arc::new(vec![("client.ping.interval".to_string(), "30".to_string())]);
        let h = handler(gc, sm, db, tc, rc, cp);
        let cmd = InternalServerCommandJoinApproved::new(
            0, String::new(), "Coach".to_string(), String::new(), "REPLAY".to_string(), vec![],
        );
        let (tx2, _rx2) = mpsc::unbounded_channel();
        assert!(h.handle_command(&cmd, 4, tx2).await);
        let msg = rx.try_recv().expect("expected a serverUserSettings message");
        assert!(msg.contains("serverUserSettings"));
    }

    #[test]
    fn parse_client_mode_recognizes_all_variants() {
        assert_eq!(parse_client_mode("PLAYER"), Some(ClientMode::PLAYER));
        assert_eq!(parse_client_mode("spectator"), Some(ClientMode::SPECTATOR));
        assert_eq!(parse_client_mode("Replay"), Some(ClientMode::REPLAY));
        assert_eq!(parse_client_mode("bogus"), None);
    }
}
