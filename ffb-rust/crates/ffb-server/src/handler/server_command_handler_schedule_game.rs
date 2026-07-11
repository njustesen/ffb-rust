/// 1:1 translation of com.fumbbl.ffb.server.handler.ServerCommandHandlerScheduleGame.
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use ffb_model::enums::{NetCommandId, Rules};
use crate::game_cache::GameCache;
use crate::net::commands::internal_server_command_schedule_game::InternalServerCommandScheduleGame;
use crate::roster_cache::RosterCache;
use crate::team_cache::TeamCache;

pub struct ServerCommandHandlerScheduleGame {
    game_cache: Arc<Mutex<GameCache>>,
    team_cache: Arc<TeamCache>,
    roster_cache: Arc<RosterCache>,
}

impl ServerCommandHandlerScheduleGame {
    pub fn new(game_cache: Arc<Mutex<GameCache>>, team_cache: Arc<TeamCache>, roster_cache: Arc<RosterCache>) -> Self {
        Self { game_cache, team_cache, roster_cache }
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
    ///   `ServerRequestProcessor` — no `ServerMode`/`FumbblRequestLoadTeam`/request-processor
    ///   dispatch exists in this crate yet, so that branch remains a narrow, separately-scoped
    ///   gap (unrelated to roster resolution — see `ServerCommandHandlerUploadGame`'s HTTP
    ///   backup-request branch for the same class of gap).
    /// - Standalone mode synchronously looks up `Team` objects via `GameCache.getTeamById`
    ///   (now real, Phase ZY.2/ZY.3) and attaches them.
    ///
    /// Java attaches the two teams incrementally onto an already-existing (team-less) `Game`
    /// via two `GameCache.addTeamToGame` calls; this crate's `Game`/`DriverGameState` require
    /// both teams up front at construction (no empty/skeleton `Game` state exists here — see
    /// `GameState::start_game`'s doc comment), so both resolved teams are passed to
    /// `GameState::start_game` in one call instead — the same substitution this crate's other
    /// team-attaching handlers already make (e.g. `ServerCommandHandlerAddLoadedTeam`'s test
    /// fixtures, `ServerCommandHandlerFumbblGameChecked`).
    fn load_teams(&self, game_id: i64, cmd: &InternalServerCommandScheduleGame) {
        let team_home = GameCache::get_team_by_id(cmd.get_team_home_id(), &self.team_cache, &self.roster_cache);
        let team_away = GameCache::get_team_by_id(cmd.get_team_away_id(), &self.team_cache, &self.roster_cache);
        let (Ok(team_home), Ok(team_away)) = (team_home, team_away) else {
            log::error!(
                "game {}: unable to resolve teams for ScheduleGame (home={}, away={})",
                game_id, cmd.get_team_home_id(), cmd.get_team_away_id()
            );
            return;
        };

        let mut gc = self.game_cache.lock().unwrap();
        if let Some(gs) = gc.get_game_state_by_id_mut(game_id) {
            gs.start_game(team_home, team_away, Rules::Bb2025, seed());
        }
    }
}

/// Java has no equivalent explicit-seed concept (its RNG is unseeded `java.util.Random`);
/// this crate's engine requires a `u64` seed, so a wall-clock-derived value stands in,
/// matching the convention already used by `server_command_handler_ping.rs`/
/// `server_network_entropy_task.rs` elsewhere in this crate.
fn seed() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(0)
}

impl Default for ServerCommandHandlerScheduleGame {
    fn default() -> Self {
        Self::new(
            Arc::new(Mutex::new(GameCache::new())),
            Arc::new(TeamCache::new()),
            Arc::new(RosterCache::new()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn scratch_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("ffb_schedule_game_test_{}", name));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

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
        let h = ServerCommandHandlerScheduleGame::new(
            Arc::clone(&gc), Arc::new(TeamCache::new()), Arc::new(RosterCache::new()),
        );
        let game_id = h.create_scheduled_game();
        assert!(gc.lock().unwrap().get_game_state_by_id(game_id).is_some());
    }

    #[test]
    fn create_scheduled_game_returns_distinct_ids() {
        let h = ServerCommandHandlerScheduleGame::default();
        let a = h.create_scheduled_game();
        let b = h.create_scheduled_game();
        assert_ne!(a, b);
    }

    #[test]
    fn load_teams_with_unresolvable_teams_leaves_game_unstarted() {
        let gc = Arc::new(Mutex::new(GameCache::new()));
        let h = ServerCommandHandlerScheduleGame::new(
            Arc::clone(&gc), Arc::new(TeamCache::new()), Arc::new(RosterCache::new()),
        );
        let game_id = h.create_scheduled_game();
        let cmd = InternalServerCommandScheduleGame::new("home".to_string(), "away".to_string());
        h.load_teams(game_id, &cmd);
        // No team/roster files were ever loaded, so the game slot exists but was never started.
        let gc = gc.lock().unwrap();
        let gs = gc.get_game_state_by_id(game_id).unwrap();
        assert!(!gs.is_started());
    }

    #[test]
    fn handle_command_resolves_teams_and_starts_game() {
        let team_dir = scratch_dir("teams");
        let roster_dir = scratch_dir("rosters");
        fs::write(
            team_dir.join("team_home.xml"),
            r#"<team id="home1"><name>Home Team</name><rosterId>human</rosterId><coach>Alice</coach><reRolls>1</reRolls></team>"#,
        ).unwrap();
        fs::write(
            team_dir.join("team_away.xml"),
            r#"<team id="away1"><name>Away Team</name><rosterId>human</rosterId><coach>Bob</coach></team>"#,
        ).unwrap();
        fs::write(
            roster_dir.join("roster_human.xml"),
            r#"<roster id="human"><name>Human</name><reRollCost>50000</reRollCost></roster>"#,
        ).unwrap();

        let mut team_cache = TeamCache::new();
        team_cache.init(&team_dir).unwrap();
        let mut roster_cache = RosterCache::new();
        roster_cache.init(&roster_dir).unwrap();

        let gc = Arc::new(Mutex::new(GameCache::new()));
        let h = ServerCommandHandlerScheduleGame::new(
            Arc::clone(&gc), Arc::new(team_cache), Arc::new(roster_cache),
        );
        let game_id = h.create_scheduled_game();
        let cmd = InternalServerCommandScheduleGame::new("home1".to_string(), "away1".to_string());
        h.load_teams(game_id, &cmd);

        let gc = gc.lock().unwrap();
        let gs = gc.get_game_state_by_id(game_id).unwrap();
        assert!(gs.is_started());
        let game = gs.get_game().unwrap();
        assert_eq!(game.team_home.name, "Home Team");
        assert_eq!(game.team_away.name, "Away Team");
        assert_eq!(game.team_home.roster_id, "human");
        assert_eq!(game.team_home.team_value, 50_000);

        let _ = fs::remove_dir_all(&team_dir);
        let _ = fs::remove_dir_all(&roster_dir);
    }
}
