use std::collections::HashMap;
use std::path::PathBuf;

/// File-backed roster cache for STANDALONE server mode — 1:1 translation of Java RosterCache.
pub struct RosterCache {
    roster_file_by_roster_id: HashMap<String, PathBuf>,
    roster_file_by_team_id: HashMap<String, PathBuf>,
}

impl RosterCache {
    pub fn new() -> Self {
        Self {
            roster_file_by_roster_id: HashMap::new(),
            roster_file_by_team_id: HashMap::new(),
        }
    }

    pub fn init(&mut self, rosters_dir: &std::path::Path) -> std::io::Result<()> {
        // Phase ZU: scan directory and populate roster_file_by_roster_id / roster_file_by_team_id
        todo!("Phase ZU: XML roster file parsing")
    }

    pub fn get_roster_file_by_roster_id(&self, roster_id: &str) -> Option<&PathBuf> {
        self.roster_file_by_roster_id.get(roster_id)
    }

    pub fn get_roster_file_by_team_id(&self, team_id: &str) -> Option<&PathBuf> {
        self.roster_file_by_team_id.get(team_id)
    }

    pub fn roster_count(&self) -> usize {
        self.roster_file_by_roster_id.len()
    }
}

impl Default for RosterCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_cache_is_empty() {
        let cache = RosterCache::new();
        assert_eq!(cache.roster_count(), 0);
    }

    #[test]
    fn test_missing_roster_returns_none() {
        let cache = RosterCache::new();
        assert!(cache.get_roster_file_by_roster_id("unknown").is_none());
    }
}
