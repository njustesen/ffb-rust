use std::collections::HashMap;

/// In-memory cache of active replay sessions — 1:1 translation of Java ReplayCache.
pub struct ReplayCache {
    states_by_name: HashMap<String, ReplayState>,
}

/// Minimal replay state record (full implementation in Phase ZU).
pub struct ReplayState {
    name: String,
}

impl ReplayState {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }
}

impl ReplayCache {
    pub fn new() -> Self {
        Self { states_by_name: HashMap::new() }
    }

    pub fn replay_state(&self, name: &str) -> Option<&ReplayState> {
        self.states_by_name.get(name)
    }

    pub fn add(&mut self, state: ReplayState) -> bool {
        let name = state.name.clone();
        self.states_by_name.insert(name, state).is_none()
    }

    pub fn close_replay(&mut self, replay_name: &str) {
        if !replay_name.is_empty() {
            self.states_by_name.remove(replay_name);
        }
    }

    pub fn replay_count(&self) -> usize {
        self.states_by_name.len()
    }
}

impl Default for ReplayCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_retrieve() {
        let mut cache = ReplayCache::new();
        cache.add(ReplayState::new("game_42"));
        assert!(cache.replay_state("game_42").is_some());
        assert_eq!(cache.replay_count(), 1);
    }

    #[test]
    fn test_close_replay() {
        let mut cache = ReplayCache::new();
        cache.add(ReplayState::new("game_1"));
        cache.close_replay("game_1");
        assert!(cache.replay_state("game_1").is_none());
    }
}
