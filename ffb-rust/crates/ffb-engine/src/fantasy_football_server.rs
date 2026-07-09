use std::collections::HashMap;

use crate::db_updater::DbUpdater;
use crate::debug_log::DebugLog;
use crate::game_cache::GameCache;
use crate::replay_cache::ReplayCache;
use crate::server_replayer::ServerReplayer;
use crate::server_sketch_manager::ServerSketchManager;
use crate::util::rng::fortuna::Fortuna;

/// Top-level FFB server — 1:1 translation of Java FantasyFootballServer.
///
/// Owns all server subsystems: caches, network, DB updater, RNG, debug log.
/// Networking and DB wiring are deferred to Phase ZU.
pub struct FantasyFootballServer {
    mode: ServerMode,
    properties: HashMap<String, String>,
    pub debug_log: DebugLog,
    pub game_cache: GameCache,
    pub replay_cache: ReplayCache,
    pub db_updater: DbUpdater,
    pub replayer: ServerReplayer,
    pub sketch_manager: ServerSketchManager,
    pub fortuna: Fortuna,
    blocking_new_games: bool,
}

/// Server operating mode — 1:1 translation of Java ServerMode enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerMode {
    Standalone,
    Fumbbl,
}

impl FantasyFootballServer {
    pub fn new(mode: ServerMode) -> Self {
        Self {
            mode,
            properties: HashMap::new(),
            debug_log: DebugLog::default(),
            game_cache: GameCache::new(),
            replay_cache: ReplayCache::new(),
            db_updater: DbUpdater::new(),
            replayer: ServerReplayer::new(),
            sketch_manager: ServerSketchManager::new(),
            fortuna: Fortuna::new(),
            blocking_new_games: false,
        }
    }

    pub fn get_mode(&self) -> ServerMode {
        self.mode
    }

    pub fn get_property(&self, key: &str) -> &str {
        self.properties.get(key).map(|s| s.as_str()).unwrap_or("")
    }

    pub fn set_property(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.properties.insert(key.into(), value.into());
    }

    pub fn is_blocking_new_games(&self) -> bool {
        self.blocking_new_games
    }

    pub fn set_blocking_new_games(&mut self, blocking: bool) {
        self.blocking_new_games = blocking;
    }

    pub fn start(&mut self) {
        // Phase ZU: start WebSocket server, DB connection, entropy server
        todo!("Phase ZU: server startup")
    }

    pub fn stop(&mut self) {
        self.db_updater.stop();
        self.replayer.stop();
    }
}

impl Default for FantasyFootballServer {
    fn default() -> Self {
        Self::new(ServerMode::Standalone)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_server_standalone() {
        let server = FantasyFootballServer::new(ServerMode::Standalone);
        assert_eq!(server.get_mode(), ServerMode::Standalone);
        assert!(!server.is_blocking_new_games());
    }

    #[test]
    fn test_set_property() {
        let mut server = FantasyFootballServer::new(ServerMode::Standalone);
        server.set_property("server.port", "8080");
        assert_eq!(server.get_property("server.port"), "8080");
        assert_eq!(server.get_property("missing"), "");
    }
}
