/// 1:1 translation of com.fumbbl.ffb.server.net.commands.InternalServerCommandSocketClosed.
use super::internal_server_command::InternalServerCommand;

pub struct InternalServerCommandSocketClosed;

impl InternalServerCommandSocketClosed {
    pub fn new() -> Self {
        Self
    }
}

impl Default for InternalServerCommandSocketClosed {
    fn default() -> Self {
        Self::new()
    }
}

impl InternalServerCommand for InternalServerCommandSocketClosed {
    fn get_id(&self) -> &'static str {
        "internalServerSocketClosed"
    }

    fn get_game_id(&self) -> i64 {
        0
    }
}
