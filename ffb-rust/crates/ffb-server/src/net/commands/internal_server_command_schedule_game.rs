/// 1:1 translation of com.fumbbl.ffb.server.net.commands.InternalServerCommandScheduleGame.
use super::internal_server_command::InternalServerCommand;

pub struct InternalServerCommandScheduleGame {
    pub team_home_id: String,
    pub team_away_id: String,
}

impl InternalServerCommandScheduleGame {
    pub fn new(team_home_id: String, team_away_id: String) -> Self {
        Self { team_home_id, team_away_id }
    }

    pub fn get_team_home_id(&self) -> &str {
        &self.team_home_id
    }

    pub fn get_team_away_id(&self) -> &str {
        &self.team_away_id
    }
}

impl InternalServerCommand for InternalServerCommandScheduleGame {
    fn get_id(&self) -> &'static str {
        "internalServerScheduleGame"
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
        let _ = InternalServerCommandScheduleGame::new("home".to_string(), "away".to_string());
    }

    #[test]
    fn get_id() {
        let c = InternalServerCommandScheduleGame::new("h".to_string(), "a".to_string());
        assert_eq!(c.get_id(), "internalServerScheduleGame");
    }

    #[test]
    fn get_team_home_id() {
        let c = InternalServerCommandScheduleGame::new("homeTeam".to_string(), "a".to_string());
        assert_eq!(c.get_team_home_id(), "homeTeam");
    }

    #[test]
    fn get_team_away_id() {
        let c = InternalServerCommandScheduleGame::new("h".to_string(), "awayTeam".to_string());
        assert_eq!(c.get_team_away_id(), "awayTeam");
    }

    #[test]
    fn is_internal() {
        let c = InternalServerCommandScheduleGame::new("h".to_string(), "a".to_string());
        assert!(c.is_internal());
    }
}
