use std::collections::HashMap;

use crate::game_state::GameState;

/// In-memory map of active game states, keyed by game ID — 1:1 translation of Java GameCache.
pub struct GameCache {
    game_state_by_id: HashMap<i64, GameState>,
    game_id_by_name: HashMap<String, i64>,
}

impl GameCache {
    pub fn new() -> Self {
        Self {
            game_state_by_id: HashMap::new(),
            game_id_by_name: HashMap::new(),
        }
    }

    pub fn get_game_state_by_id(&self, game_id: i64) -> Option<&GameState> {
        self.game_state_by_id.get(&game_id)
    }

    pub fn get_game_state_by_id_mut(&mut self, game_id: i64) -> Option<&mut GameState> {
        self.game_state_by_id.get_mut(&game_id)
    }

    pub fn all_game_states(&self) -> Vec<&GameState> {
        self.game_state_by_id.values().collect()
    }

    pub fn add_game(&mut self, game_state: GameState, game_id: i64) {
        self.game_state_by_id.insert(game_id, game_state);
    }

    pub fn get_game_state_by_name(&self, game_name: &str) -> Option<&GameState> {
        let game_id = self.game_id_by_name.get(game_name)?;
        self.game_state_by_id.get(game_id)
    }

    pub fn get_game_name(&self, id: i64) -> Option<&str> {
        self.game_id_by_name
            .iter()
            .find(|(_, &v)| v == id)
            .map(|(k, _)| k.as_str())
    }

    pub fn map_game_name_to_id(&mut self, game_name: impl Into<String>, game_id: i64) {
        self.game_id_by_name.insert(game_name.into(), game_id);
    }

    pub fn remove_mapping_for_game_id(&mut self, game_id: i64) {
        self.game_id_by_name.retain(|_, &mut v| v != game_id);
    }

    pub fn remove_game(&mut self, game_id: i64) -> Option<GameState> {
        self.remove_mapping_for_game_id(game_id);
        self.game_state_by_id.remove(&game_id)
    }

    pub fn close_all_games(&mut self) {
        let ids: Vec<i64> = self.game_state_by_id.keys().copied().collect();
        for id in ids {
            self.remove_game(id);
        }
    }

    pub fn game_count(&self) -> usize {
        self.game_state_by_id.len()
    }
}

impl Default for GameCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_cache() {
        let cache = GameCache::new();
        assert_eq!(cache.game_count(), 0);
        assert!(cache.get_game_state_by_id(1).is_none());
    }

    #[test]
    fn test_add_and_retrieve() {
        let mut cache = GameCache::new();
        cache.add_game(GameState::new(), 42);
        assert_eq!(cache.game_count(), 1);
        assert!(cache.get_game_state_by_id(42).is_some());
    }
}
