/// 1:1 translation of com.fumbbl.ffb.server.net.commands.InternalServerCommandDeleteGame.
use super::internal_server_command::InternalServerCommand;

pub struct InternalServerCommandDeleteGame {
    pub game_id: i64,
    pub with_games_info: bool,
}

impl InternalServerCommandDeleteGame {
    pub fn new(game_id: i64, with_games_info: bool) -> Self {
        Self { game_id, with_games_info }
    }

    pub fn is_with_games_info(&self) -> bool {
        self.with_games_info
    }
}

impl InternalServerCommand for InternalServerCommandDeleteGame {
    fn get_id(&self) -> &'static str {
        "internalServerDeleteGame"
    }

    fn get_game_id(&self) -> i64 {
        self.game_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let _ = InternalServerCommandDeleteGame::new(42, true);
    }

    #[test]
    fn get_id() {
        let c = InternalServerCommandDeleteGame::new(1, false);
        assert_eq!(c.get_id(), "internalServerDeleteGame");
    }

    #[test]
    fn get_game_id() {
        let c = InternalServerCommandDeleteGame::new(7, false);
        assert_eq!(c.get_game_id(), 7);
    }

    #[test]
    fn is_with_games_info() {
        let c = InternalServerCommandDeleteGame::new(1, true);
        assert!(c.is_with_games_info());
    }

    #[test]
    fn is_internal() {
        let c = InternalServerCommandDeleteGame::new(1, false);
        assert!(c.is_internal());
    }
}
