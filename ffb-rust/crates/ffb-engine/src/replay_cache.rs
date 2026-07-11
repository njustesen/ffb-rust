use std::collections::HashMap;

use crate::replay_state::ReplayState;

/// In-memory cache of active replay sessions — 1:1 translation of Java ReplayCache.
///
/// Java's `add(ReplayState)` also does session-driven cache eviction (closing any other
/// cached replay with no connected sessions left, via `server.getReplaySessionManager()`);
/// that eviction sweep needs a live session registry to query and is out of scope for this
/// crate (`ffb-engine` has no session/networking layer — see `ServerReplayer`'s own doc
/// comment on the same crate boundary). `add`/`replay_state`/`close_replay` themselves are
/// otherwise ported for real.
pub struct ReplayCache {
    states_by_name: HashMap<String, ReplayState>,
}

impl ReplayCache {
    pub fn new() -> Self {
        Self { states_by_name: HashMap::new() }
    }

    pub fn replay_state(&self, name: &str) -> Option<&ReplayState> {
        self.states_by_name.get(name)
    }

    pub fn replay_state_mut(&mut self, name: &str) -> Option<&mut ReplayState> {
        self.states_by_name.get_mut(name)
    }

    /// Java: `add(ReplayState)`. Returns `true` if this is a newly-added
    /// replay name (Java: `previousState == null`), matching the previous
    /// stub's return convention.
    pub fn add(&mut self, state: ReplayState) -> bool {
        let name = state.get_name().to_string();
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

    #[test]
    fn add_existing_name_returns_false() {
        let mut cache = ReplayCache::new();
        assert!(cache.add(ReplayState::new("game_1")));
        assert!(!cache.add(ReplayState::new("game_1")));
    }

    #[test]
    fn replay_state_mut_allows_updates() {
        let mut cache = ReplayCache::new();
        cache.add(ReplayState::new("game_1"));
        cache.replay_state_mut("game_1").unwrap().prevent_coach_from_sketching("Alice");
        assert!(cache.replay_state("game_1").unwrap().is_coach_prevented_from_sketching("Alice"));
    }
}
