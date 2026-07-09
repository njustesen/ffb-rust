use std::collections::HashMap;
use std::path::PathBuf;

/// File-backed team cache for STANDALONE server mode — 1:1 translation of Java TeamCache.
pub struct TeamCache {
    team_files: HashMap<String, PathBuf>,
}

impl TeamCache {
    pub fn new() -> Self {
        Self { team_files: HashMap::new() }
    }

    pub fn init(&mut self, teams_dir: &std::path::Path) -> std::io::Result<()> {
        // Phase ZU: scan directory and populate team_files by team ID
        todo!("Phase ZU: XML team file parsing")
    }

    pub fn get_team_by_id(&self, team_id: &str) -> Option<&PathBuf> {
        self.team_files.get(team_id)
    }

    pub fn get_teams_for_coach(&self, coach: &str) -> Vec<&PathBuf> {
        // Phase ZU: filter team_files by coach field in XML
        todo!("Phase ZU: coach-filtered team lookup")
    }

    pub fn team_count(&self) -> usize {
        self.team_files.len()
    }
}

impl Default for TeamCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_cache_empty() {
        let cache = TeamCache::new();
        assert_eq!(cache.team_count(), 0);
    }

    #[test]
    fn test_missing_team_returns_none() {
        let cache = TeamCache::new();
        assert!(cache.get_team_by_id("999").is_none());
    }
}
