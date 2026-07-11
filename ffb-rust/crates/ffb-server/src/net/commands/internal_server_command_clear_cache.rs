/// 1:1 translation of com.fumbbl.ffb.server.net.commands.InternalServerCommandClearCache.
use super::internal_server_command::InternalServerCommand;

pub struct InternalServerCommandClearCache;

impl InternalServerCommandClearCache {
    pub fn new() -> Self {
        Self
    }
}

impl Default for InternalServerCommandClearCache {
    fn default() -> Self {
        Self::new()
    }
}

impl InternalServerCommand for InternalServerCommandClearCache {
    fn get_id(&self) -> &'static str {
        "internalServerClearCache"
    }

    fn get_game_id(&self) -> i64 {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let _ = InternalServerCommandClearCache::new();
    }

    #[test]
    fn get_id() {
        let c = InternalServerCommandClearCache::new();
        assert_eq!(c.get_id(), "internalServerClearCache");
    }

    #[test]
    fn get_game_id() {
        let c = InternalServerCommandClearCache::new();
        assert_eq!(c.get_game_id(), 0);
    }

    #[test]
    fn is_internal() {
        let c = InternalServerCommandClearCache::new();
        assert!(c.is_internal());
    }

    #[test]
    fn default() {
        let _ = InternalServerCommandClearCache::default();
    }
}
