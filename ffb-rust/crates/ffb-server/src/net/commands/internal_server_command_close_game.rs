/// 1:1 translation of com.fumbbl.ffb.server.net.commands.InternalServerCommandCloseGame.
use super::internal_server_command::InternalServerCommand;

pub struct InternalServerCommandCloseGame {
    pub game_id: i64,
}

impl InternalServerCommandCloseGame {
    pub fn new(game_id: i64) -> Self {
        Self { game_id }
    }
}

impl InternalServerCommand for InternalServerCommandCloseGame {
    fn get_id(&self) -> &'static str {
        "internalServerCloseGame"
    }

    fn get_game_id(&self) -> i64 {
        self.game_id
    }
}
