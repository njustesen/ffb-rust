/// 1:1 translation of com.fumbbl.ffb.server.net.commands.InternalServerCommandUploadGame.
use super::internal_server_command::InternalServerCommand;

pub struct InternalServerCommandUploadGame {
    pub game_id: i64,
    pub conceding_team_id: Option<String>,
}

impl InternalServerCommandUploadGame {
    pub fn new(game_id: i64) -> Self {
        Self { game_id, conceding_team_id: None }
    }

    pub fn new_with_conceding(game_id: i64, conceding_team_id: Option<String>) -> Self {
        Self { game_id, conceding_team_id }
    }

    pub fn get_conceding_team_id(&self) -> Option<&str> {
        self.conceding_team_id.as_deref()
    }
}

impl InternalServerCommand for InternalServerCommandUploadGame {
    fn get_id(&self) -> &'static str {
        "internalServerUploadGame"
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
        let _ = InternalServerCommandUploadGame::new(42);
    }

    #[test]
    fn construct_with_conceding() {
        let c = InternalServerCommandUploadGame::new_with_conceding(1, Some("team1".to_string()));
        assert_eq!(c.get_conceding_team_id(), Some("team1"));
    }

    #[test]
    fn get_id() {
        let c = InternalServerCommandUploadGame::new(1);
        assert_eq!(c.get_id(), "internalServerUploadGame");
    }

    #[test]
    fn get_game_id() {
        let c = InternalServerCommandUploadGame::new(7);
        assert_eq!(c.get_game_id(), 7);
    }

    #[test]
    fn is_internal() {
        let c = InternalServerCommandUploadGame::new(1);
        assert!(c.is_internal());
    }

    #[test]
    fn no_conceding_team_by_default() {
        let c = InternalServerCommandUploadGame::new(1);
        assert!(c.get_conceding_team_id().is_none());
    }
}
