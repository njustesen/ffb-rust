/// 1:1 translation of com.fumbbl.ffb.server.GameCache (in-memory MVP only — no DB).
use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use crate::game_state::GameState;

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
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
