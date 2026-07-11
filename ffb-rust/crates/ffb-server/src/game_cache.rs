/// 1:1 translation of com.fumbbl.ffb.server.GameCache (in-memory MVP only — no DB).
use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};

use ffb_model::enums::{PlayerState, SendToBoxReason, PS_MISSING, PS_RESERVE};
use ffb_model::model::game::Game;
use ffb_model::model::team::Team;
use ffb_model::util::util_box::UtilBox;
use mysql_async::Error as DbError;

use crate::db::db_connection_manager::DbConnectionManager;
use crate::db::delete::db_games_info_delete::DbGamesInfoDelete;
use crate::db::delete::db_games_serialized_delete::DbGamesSerializedDelete;
use crate::db::delete::db_player_markers_delete_parameter::DbPlayerMarkersDeleteParameter;
use crate::db::insert::db_player_markers_insert_parameter_list::DbPlayerMarkersInsertParameterList;
use crate::db::query::db_games_serialized_query::DbGamesSerializedQuery;
use crate::db::query::db_user_settings_query::DbUserSettingsQuery;
use crate::db::update::db_games_info_update::DbGamesInfoUpdate;
use crate::db::update::db_games_info_update_parameter::DbGamesInfoUpdateParameter;
use crate::db::update::db_games_serialized_update::DbGamesSerializedUpdate;
use crate::db::update::db_games_serialized_update_parameter::DbGamesSerializedUpdateParameter;
use crate::game_state::GameState;
use crate::net::server_communication::ServerCommunication;
use crate::net::session_manager::SessionManager;

static NEXT_GAME_ID: AtomicI64 = AtomicI64::new(1);

fn next_game_id() -> i64 {
    NEXT_GAME_ID.fetch_add(1, Ordering::Relaxed)
}

/// In-memory game cache.  The Java version backs this with a database; the
/// Rust MVP uses `HashMap`s only.
///
/// Java: `GameCache`
pub struct GameCache {
    /// Java: `fGameStateById`
    game_state_by_id: HashMap<i64, GameState>,
    /// Java: `fGameIdByName`
    game_id_by_name: HashMap<String, i64>,
}

impl GameCache {
    /// Java: `new GameCache(server)`
    pub fn new() -> Self {
        Self {
            game_state_by_id: HashMap::new(),
            game_id_by_name: HashMap::new(),
        }
    }

    /// Java: `getGameStateById(long)`
    pub fn get_game_state_by_id(&self, game_id: i64) -> Option<&GameState> {
        self.game_state_by_id.get(&game_id)
    }

    /// Java: `getGameStateById(long)` — mutable
    pub fn get_game_state_by_id_mut(&mut self, game_id: i64) -> Option<&mut GameState> {
        self.game_state_by_id.get_mut(&game_id)
    }

    /// Java: `getGameStateByName(String)`
    pub fn get_game_state_by_name(&self, game_name: &str) -> Option<&GameState> {
        let id = self.game_id_by_name.get(game_name)?;
        self.game_state_by_id.get(id)
    }

    /// Java: `getGameStateByName` — mutable
    pub fn get_game_state_by_name_mut(&mut self, game_name: &str) -> Option<&mut GameState> {
        let id = *self.game_id_by_name.get(game_name)?;
        self.game_state_by_id.get_mut(&id)
    }

    /// Java: `createGameState(GameStartMode)` — creates a new, empty game slot.
    pub fn create_game_state(&mut self) -> i64 {
        let game_id = next_game_id();
        let gs = GameState::new(game_id);
        self.game_state_by_id.insert(game_id, gs);
        game_id
    }

    /// Java: `mapGameNameToId(String, long)`
    pub fn map_game_name_to_id(&mut self, game_name: String, game_id: i64) {
        self.game_id_by_name.insert(game_name, game_id);
    }

    /// Java: `addGame(GameState)` — register a game that already has an ID.
    pub fn add_game(&mut self, game_state: GameState) {
        self.game_state_by_id.insert(game_state.get_id(), game_state);
    }

    /// Java: `allGameStates()`
    pub fn all_game_ids(&self) -> Vec<i64> {
        self.game_state_by_id.keys().copied().collect()
    }

    /// Java: `removeMappingForGameId(long)`.
    fn remove_mapping_for_game_id(&mut self, game_id: i64) {
        let game_name = self
            .game_id_by_name
            .iter()
            .find(|(_, &id)| id == game_id)
            .map(|(name, _)| name.clone());
        if let Some(name) = game_name {
            self.game_id_by_name.remove(&name);
        }
    }

    /// Java: `private removeGame(long gameId)`.
    ///
    /// Java also decides whether to `queueDbDelete` based on the game never having
    /// finished joining, or being scheduled-but-not-started, or being a test game.
    /// This crate's `Game` model has no `scheduled`/`started` date fields (see
    /// `ffb-model/src/model/game.rs`), so the "hasn't started" half of Java's OR
    /// condition can't be evaluated here; the team-not-joined and `isTesting()`
    /// checks are ported faithfully. Returns `Some(should_queue_db_delete)` if a
    /// game was actually removed, `None` if there was nothing cached under `game_id`.
    fn remove_game(&mut self, game_id: i64) -> Option<bool> {
        let cached = self.game_state_by_id.remove(&game_id)?;
        self.remove_mapping_for_game_id(cached.get_id());

        let should_delete = match cached.get_game() {
            Some(game) => {
                game.team_home.id.is_empty() || game.team_away.id.is_empty() || game.testing
            }
            // Java: a GameState with no Game at all has certainly never started.
            None => true,
        };
        Some(should_delete)
    }

    /// Java: `closeGame(long gameId)`.
    ///
    /// Java reaches `SessionManager`/`ServerCommunication`/`DbUpdater`/`RequestProcessor`
    /// via `getServer().getX()`; per this crate's convention (see module doc comment on
    /// `roster_cache.rs`, `server_command_handler_password_challenge.rs`) they are
    /// threaded through explicitly instead. The `FumbblRequestRemoveGamestate`/
    /// `fServer.closeResources(gameId)` tail (mode-dependent cleanup) has no Rust
    /// equivalent yet — same documented gap as elsewhere in this phase — so this performs
    /// the real, portable half: closing every session tracking the game and removing it
    /// from the cache (returning whether a DB delete should be queued, mirroring
    /// `remove_game`).
    pub fn close_game(
        &mut self,
        game_id: i64,
        session_manager: &SessionManager,
        communication: &ServerCommunication,
    ) -> Option<bool> {
        if game_id <= 0 {
            return None;
        }
        if self.game_state_by_id.get(&game_id).is_none() {
            return None;
        }
        for session_id in session_manager.get_sessions_for_game_id(game_id) {
            communication.close(session_id);
        }
        self.remove_game(game_id)
    }

    /// Java: `addTeamToGame(GameState, Team, boolean)`.
    ///
    /// Java's `player instanceof ZappedPlayer` / `player instanceof RosterPlayer`
    /// reconciliation has no direct equivalent here: this codebase's `Player` is a single
    /// unified struct (with a `zapped: bool` flag) rather than Java's two subtypes, and
    /// zap application/restoration already happens via `Game::add_zapped_player` /
    /// `Game::remove_zapped_player` (see `ffb-model/src/model/game.rs`) at the point a
    /// card effect actually zaps/restores a player — a `Team`'s `players` list is never
    /// split across two subtypes here the way Java's is, so there is nothing left to
    /// reconcile in this method. The remaining logic (swap team into the field model,
    /// set RESERVE/MISSING+MNG state, box placement, copy `currentSpps`) is ported below.
    ///
    /// Callers are responsible for the trailing `queueDbUpdate(gameState, true)` (see
    /// `GameCache::queue_db_update`), matching this crate's convention of threading DB
    /// access through explicitly rather than reaching a `getServer()` singleton.
    pub fn add_team_to_game(game: &mut Game, team: Team, home_team: bool) {
        let old_ids: Vec<String> = if home_team {
            game.team_home.players.iter().map(|p| p.id.clone()).collect()
        } else {
            game.team_away.players.iter().map(|p| p.id.clone()).collect()
        };
        for id in &old_ids {
            game.field_model.remove_player(id);
        }

        if home_team {
            game.team_home = team;
        } else {
            game.team_away = team;
        }

        let players: Vec<(String, bool, i32)> = {
            let new_team = if home_team { &game.team_home } else { &game.team_away };
            new_team
                .players
                .iter()
                .map(|p| (p.id.clone(), p.recovering_injury.is_some(), p.current_spps))
                .collect()
        };

        for (player_id, has_recovering_injury, current_spps) in players {
            if has_recovering_injury {
                game.field_model.set_player_state(&player_id, PlayerState::new(PS_MISSING));
                let team_result = if home_team { &mut game.game_result.home } else { &mut game.game_result.away };
                team_result.player_result_mut(&player_id).send_to_box_reason = Some(SendToBoxReason::Mng);
            } else {
                game.field_model.set_player_state(&player_id, PlayerState::new(PS_RESERVE));
            }

            UtilBox::put_player_into_box(game, &player_id);

            if current_spps > 0 {
                let team_result = if home_team { &mut game.game_result.home } else { &mut game.game_result.away };
                team_result.player_result_mut(&player_id).current_spps = current_spps;
            }
        }
    }

    /// Java: `queueDbUpdate(GameState, boolean withSerialization)`.
    ///
    /// Java hands the built `DbTransaction` off to an async `DbUpdater` and returns
    /// immediately; there is no `DbUpdater` actor in this crate, so this performs the
    /// update inline against a connection from the given pool (same pattern already
    /// established by `ServerCommandHandlerDeleteGame::handle_command`). Degrades to a
    /// no-op when no DB pool is configured (`pool_ready() == false`), e.g. in tests.
    ///
    /// `Game` has no `scheduled`/`started`/`finished` date fields (see
    /// `add_team_to_game`'s doc comment), so `DbGamesInfoUpdateParameter`'s corresponding
    /// fields are left `None`. Serialization (`DbGamesSerializedUpdateParameter`'s gzip
    /// blob) has no GameState-to-JSON pipeline wired up yet in this crate (see
    /// `db_games_serialized_update_parameter.rs`'s own doc comment), so an empty blob is
    /// written when `with_serialization` is true — narrower than Java, but faithfully
    /// invoking the same two update statements in the same order.
    pub async fn queue_db_update(
        db_connection_manager: &DbConnectionManager,
        game: &Game,
        with_serialization: bool,
    ) -> Result<(), DbError> {
        if !db_connection_manager.pool_ready() {
            return Ok(());
        }
        let mut conn = db_connection_manager.open_db_connection().await?;

        let mut info_param = DbGamesInfoUpdateParameter::new(game.id as i64);
        info_param.coach_home = game.team_home.coach.clone();
        info_param.team_home_id = game.team_home.id.clone();
        info_param.team_home_name = game.team_home.name.clone();
        info_param.coach_away = game.team_away.coach.clone();
        info_param.team_away_id = game.team_away.id.clone();
        info_param.team_away_name = game.team_away.name.clone();
        info_param.half = game.half as i8;
        info_param.turn = game.turn_data_home.turn_nr as i8;
        info_param.home_playing = game.home_playing;
        info_param.status = game.status.name().to_string();
        info_param.testing = game.testing;
        info_param.admin_mode = game.admin_mode;
        DbGamesInfoUpdate::new().execute(&mut conn, &info_param).await?;

        if with_serialization {
            let ser_param = DbGamesSerializedUpdateParameter::new(game.id as i64);
            DbGamesSerializedUpdate::new().execute(&mut conn, &ser_param).await?;
        }

        db_connection_manager.close_db_connection(conn).await?;
        Ok(())
    }

    /// Java: `queueDbDelete(long gameStateId, boolean withGamesInfo)`.
    ///
    /// Same inline-against-a-pooled-connection pattern as `queue_db_update` (and as
    /// already established by `ServerCommandHandlerDeleteGame`).
    pub async fn queue_db_delete(
        db_connection_manager: &DbConnectionManager,
        game_state_id: i64,
        with_games_info: bool,
    ) -> Result<(), DbError> {
        if game_state_id <= 0 {
            return Ok(());
        }
        if !db_connection_manager.pool_ready() {
            return Ok(());
        }
        let mut conn = db_connection_manager.open_db_connection().await?;
        if with_games_info {
            DbGamesInfoDelete::new().execute(&mut conn, game_state_id).await?;
        }
        DbGamesSerializedDelete::new().execute(&mut conn, game_state_id).await?;
        db_connection_manager.close_db_connection(conn).await?;
        Ok(())
    }

    /// Java: `queryFromDb(long gameId)`.
    ///
    /// Java deserializes the gunzipped blob into a full `GameState`; per
    /// `DbGamesSerializedQuery`'s own documented gap, there is no gunzip-to-GameState
    /// pipeline in this crate yet, so this returns the raw gzipped blob bytes instead.
    pub async fn query_from_db(
        db_connection_manager: &DbConnectionManager,
        game_id: i64,
    ) -> Result<Option<Vec<u8>>, DbError> {
        if game_id <= 0 {
            return Ok(None);
        }
        if !db_connection_manager.pool_ready() {
            return Ok(None);
        }
        let mut conn = db_connection_manager.open_db_connection().await?;
        let result = DbGamesSerializedQuery::new().execute(&mut conn, game_id).await;
        db_connection_manager.close_db_connection(conn).await?;
        result
    }

    /// Java: `queueDbPlayerMarkersUpdate(GameState)`.
    pub async fn queue_db_player_markers_update(
        db_connection_manager: &DbConnectionManager,
        game_state: &GameState,
    ) -> Result<(), DbError> {
        let game = match game_state.get_game() {
            Some(g) => g,
            None => return Ok(()),
        };
        if !db_connection_manager.pool_ready() {
            return Ok(());
        }
        let mut conn = db_connection_manager.open_db_connection().await?;

        let mut user_settings_query = DbUserSettingsQuery::new();
        user_settings_query.execute(&mut conn, &game.team_home.coach).await?;
        let load_auto_home = user_settings_query
            .get_setting_value("player_marking_type")
            .map(|v| v.eq_ignore_ascii_case("auto"))
            .unwrap_or(false);

        user_settings_query.execute(&mut conn, &game.team_away.coach).await?;
        let load_auto_away = user_settings_query
            .get_setting_value("player_marking_type")
            .map(|v| v.eq_ignore_ascii_case("auto"))
            .unwrap_or(false);

        if !load_auto_home && !game.team_home.id.is_empty() {
            DbPlayerMarkersDeleteParameter::new(game.team_home.id.clone())
                .execute(&mut conn)
                .await?;
        }
        if !load_auto_away && !game.team_away.id.is_empty() {
            DbPlayerMarkersDeleteParameter::new(game.team_away.id.clone())
                .execute(&mut conn)
                .await?;
        }

        let mut insert_list = DbPlayerMarkersInsertParameterList::new();
        insert_list.init_from(Some(game_state), load_auto_home, load_auto_away);
        let mut parameters: Vec<_> = insert_list.get_parameters().to_vec();
        for parameter in parameters.iter_mut() {
            parameter.execute(&mut conn).await?;
        }

        db_connection_manager.close_db_connection(conn).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::player::Player;

    #[test]
    fn create_and_lookup() {
        let mut cache = GameCache::new();
        let id = cache.create_game_state();
        assert!(id > 0);
        assert!(cache.get_game_state_by_id(id).is_some());
    }

    #[test]
    fn name_mapping() {
        let mut cache = GameCache::new();
        let id = cache.create_game_state();
        cache.map_game_name_to_id("TestGame".to_string(), id);
        assert!(cache.get_game_state_by_name("TestGame").is_some());
        assert!(cache.get_game_state_by_name("Missing").is_none());
    }

    #[test]
    fn unknown_id_returns_none() {
        let cache = GameCache::new();
        assert!(cache.get_game_state_by_id(999).is_none());
    }

    fn team(id: &str, coach: &str, player_ids: &[&str]) -> Team {
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
            players: player_ids.iter().map(|id| Player { id: id.to_string(), ..Default::default() }).collect(),
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn game_with_teams(home_ids: &[&str], away_ids: &[&str]) -> Game {
        Game::new(team("home", "Kalimar", home_ids), team("away", "BattleLore", away_ids), Rules::Bb2025)
    }

    #[test]
    fn add_team_to_game_sets_reserve_state_and_boxes_players() {
        let mut game = game_with_teams(&[], &[]);
        let new_home = team("newhome", "Kalimar", &["h1", "h2"]);

        GameCache::add_team_to_game(&mut game, new_home, true);

        assert_eq!(game.team_home.id, "newhome");
        assert_eq!(game.field_model.player_state("h1").unwrap(), PlayerState::new(PS_RESERVE));
        assert_eq!(game.field_model.player_state("h2").unwrap(), PlayerState::new(PS_RESERVE));
        // Boxed players get a field coordinate assigned by UtilBox.
        assert!(game.field_model.player_coordinate("h1").is_some());
    }

    #[test]
    fn add_team_to_game_removes_old_team_players_from_field() {
        let mut game = game_with_teams(&["old1"], &[]);
        game.field_model.set_player_state("old1", PlayerState::new(PS_RESERVE));
        game.field_model.set_player_coordinate("old1", ffb_model::types::FieldCoordinate::new(1, 1));

        let new_home = team("newhome", "Kalimar", &["h1"]);
        GameCache::add_team_to_game(&mut game, new_home, true);

        assert!(game.field_model.player_state("old1").is_none());
        assert!(game.field_model.player_coordinate("old1").is_none());
    }

    #[test]
    fn add_team_to_game_sets_missing_and_mng_reason_for_recovering_injury() {
        let mut game = game_with_teams(&[], &[]);
        let mut new_away = team("newaway", "BattleLore", &["a1"]);
        new_away.players[0].recovering_injury = Some(ffb_model::enums::SeriousInjuryKind::BrokenArmPa);

        GameCache::add_team_to_game(&mut game, new_away, false);

        assert_eq!(game.field_model.player_state("a1").unwrap().base(), PS_MISSING);
        let result = game.game_result.away.player_result("a1").unwrap();
        assert_eq!(result.send_to_box_reason, Some(SendToBoxReason::Mng));
    }

    #[test]
    fn add_team_to_game_copies_current_spps_into_player_result() {
        let mut game = game_with_teams(&[], &[]);
        let mut new_home = team("newhome", "Kalimar", &["h1"]);
        new_home.players[0].current_spps = 12;

        GameCache::add_team_to_game(&mut game, new_home, true);

        let result = game.game_result.home.player_result("h1").unwrap();
        assert_eq!(result.current_spps, 12);
    }

    #[test]
    fn remove_game_reports_should_delete_for_test_game() {
        let mut cache = GameCache::new();
        let id = cache.create_game_state();
        // A cached GameState with no started Game (driver == None) is treated as
        // never-started, matching Java's null-Game-implies-never-started intent.
        assert_eq!(cache.remove_game(id), Some(true));
        assert!(cache.get_game_state_by_id(id).is_none());
    }

    #[test]
    fn remove_game_missing_id_returns_none() {
        let mut cache = GameCache::new();
        assert_eq!(cache.remove_game(999), None);
    }

    #[tokio::test]
    async fn close_game_removes_from_cache_and_reports_should_delete() {
        let mut cache = GameCache::new();
        let id = cache.create_game_state();
        let session_manager = SessionManager::new();
        let communication = ServerCommunication::new(
            std::sync::Arc::new(std::sync::Mutex::new(GameCache::new())),
            std::sync::Arc::new(std::sync::Mutex::new(SessionManager::new())),
        );

        let result = cache.close_game(id, &session_manager, &communication);
        assert_eq!(result, Some(true));
        assert!(cache.get_game_state_by_id(id).is_none());
    }

    #[tokio::test]
    async fn close_game_with_invalid_id_is_a_noop() {
        let mut cache = GameCache::new();
        let session_manager = SessionManager::new();
        let communication = ServerCommunication::new(
            std::sync::Arc::new(std::sync::Mutex::new(GameCache::new())),
            std::sync::Arc::new(std::sync::Mutex::new(SessionManager::new())),
        );
        assert_eq!(cache.close_game(0, &session_manager, &communication), None);
        assert_eq!(cache.close_game(42, &session_manager, &communication), None);
    }

    #[tokio::test]
    async fn queue_db_update_without_pool_is_a_noop() {
        let db = DbConnectionManager::new();
        let game = game_with_teams(&[], &[]);
        assert!(GameCache::queue_db_update(&db, &game, true).await.is_ok());
    }

    #[tokio::test]
    async fn queue_db_delete_without_pool_is_a_noop() {
        let db = DbConnectionManager::new();
        assert!(GameCache::queue_db_delete(&db, 42, true).await.is_ok());
    }

    #[tokio::test]
    async fn queue_db_delete_zero_id_is_a_noop() {
        let db = DbConnectionManager::new();
        assert!(GameCache::queue_db_delete(&db, 0, true).await.is_ok());
    }

    #[tokio::test]
    async fn query_from_db_without_pool_returns_none() {
        let db = DbConnectionManager::new();
        assert_eq!(GameCache::query_from_db(&db, 42).await.unwrap(), None);
    }

    #[tokio::test]
    async fn queue_db_player_markers_update_without_started_game_is_a_noop() {
        let db = DbConnectionManager::new();
        let game_state = GameState::new(1);
        assert!(GameCache::queue_db_player_markers_update(&db, &game_state).await.is_ok());
    }
}
