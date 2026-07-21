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
