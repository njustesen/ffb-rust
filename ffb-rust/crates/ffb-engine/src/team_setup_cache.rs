use std::collections::HashMap;
use std::path::PathBuf;

/// File-backed team setup (saved formation) cache — 1:1 translation of Java TeamSetupCache.
pub struct TeamSetupCache {
    setup_files: HashMap<TeamSetupKey, PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TeamSetupKey {
    team_id: String,
    name: String,
}

impl TeamSetupCache {
    pub fn new() -> Self {
        Self { setup_files: HashMap::new() }
    }

    pub fn init(&mut self, setups_dir: &std::path::Path) -> std::io::Result<()> {
        // Phase ZU: scan directory and populate setup_files
        todo!("Phase ZU: XML team setup file parsing")
    }

    pub fn get_setup(&self, team_id: &str, name: &str) -> Option<&PathBuf> {
        let key = TeamSetupKey { team_id: team_id.to_string(), name: name.to_string() };
        self.setup_files.get(&key)
    }

    pub fn setup_count(&self) -> usize {
        self.setup_files.len()
    }
}

impl Default for TeamSetupCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_cache_empty() {
        let cache = TeamSetupCache::new();
        assert_eq!(cache.setup_count(), 0);
    }

    #[test]
    fn test_missing_setup_returns_none() {
        let cache = TeamSetupCache::new();
        assert!(cache.get_setup("team1", "default").is_none());
    }
}
