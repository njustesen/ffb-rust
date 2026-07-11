/// 1:1 translation of com.fumbbl.ffb.server.handler.ServerCommandHandlerScheduleGame.
use std::sync::{Arc, Mutex};
use ffb_model::enums::NetCommandId;
use crate::game_cache::GameCache;
use crate::net::commands::internal_server_command_schedule_game::InternalServerCommandScheduleGame;

pub struct ServerCommandHandlerScheduleGame {
    game_cache: Arc<Mutex<GameCache>>,
}

impl ServerCommandHandlerScheduleGame {
    pub fn new(game_cache: Arc<Mutex<GameCache>>) -> Self {
        Self { game_cache }
    }

    /// Java: getId() — returns NetCommandId for SCHEDULE_GAME.
    pub fn get_id(&self) -> NetCommandId {
        NetCommandId::InternalServerScheduleGame
    }

    /// Java: `handleCommand(ReceivedCommand)` — handles scheduling a new game.
    ///
    /// Creates the empty game slot (Java: `GameCache.createGameState(SCHEDULE_GAME)`)
    /// for real, then delegates team-loading to `load_teams`.
    pub fn handle_command(&self, cmd: &InternalServerCommandScheduleGame) -> bool {
        let game_id = self.create_scheduled_game();
        self.load_teams(game_id, cmd);
        true
    }

    /// Java: `getServer().getGameCache().createGameState(GameStartMode.SCHEDULE_GAME)`.
    fn create_scheduled_game(&self) -> i64 {
        let mut gc = self.game_cache.lock().unwrap();
        gc.create_game_state()
    }

    /// Java branches on `ServerMode.FUMBBL == getServer().getMode()`:
    /// - FUMBBL mode enqueues two `FumbblRequestLoadTeam` HTTP requests on the
    ///   `ServerRequestProcessor`.
    /// - Standalone mode synchronously looks up `Team` objects via
    ///   `GameCache.getTeamById` (roster-file I/O) and attaches them with
    ///   `GameCache.addTeamToGame`.
    ///
    /// Neither `ServerMode`, `FumbblRequestLoadTeam`, nor
    /// `GameCache.getTeamById`/`addTeamToGame` exist in the Rust MVP yet.
    fn load_teams(&self, _game_id: i64, cmd: &InternalServerCommandScheduleGame) {
        todo!(
            "Phase ZV: FumbblRequestLoadTeam (HTTP) / GameCache.getTeamById+addTeamToGame (roster I/O) need wiring for team_home_id={} team_away_id={}",
            cmd.get_team_home_id(),
            cmd.get_team_away_id()
        )
    }
}

impl Default for ServerCommandHandlerScheduleGame {
    fn default() -> Self {
        Self::new(Arc::new(Mutex::new(GameCache::new())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let _ = ServerCommandHandlerScheduleGame::default();
    }

    #[test]
    fn get_id_returns_internal_server_schedule_game() {
        let h = ServerCommandHandlerScheduleGame::default();
        assert_eq!(h.get_id(), NetCommandId::InternalServerScheduleGame);
    }

    #[test]
    fn create_scheduled_game_registers_a_new_game_in_the_cache() {
        let gc = Arc::new(Mutex::new(GameCache::new()));
        let h = ServerCommandHandlerScheduleGame::new(Arc::clone(&gc));
        let game_id = h.create_scheduled_game();
        assert!(gc.lock().unwrap().get_game_state_by_id(game_id).is_some());
    }

    #[test]
    fn create_scheduled_game_returns_distinct_ids() {
        let gc = Arc::new(Mutex::new(GameCache::new()));
        let h = ServerCommandHandlerScheduleGame::new(gc);
        let a = h.create_scheduled_game();
        let b = h.create_scheduled_game();
        assert_ne!(a, b);
    }

    #[test]
    fn handle_command_hits_team_loading_todo() {
        let h = ServerCommandHandlerScheduleGame::default();
        let cmd = InternalServerCommandScheduleGame::new("home".to_string(), "away".to_string());
        let result = std::panic::catch_unwind(|| h.handle_command(&cmd));
        assert!(result.is_err(), "team loading requires FUMBBL HTTP or roster I/O wiring (narrow todo!)");
    }
}
