/// 1:1 translation of com.fumbbl.ffb.server.net.commands.InternalServerCommandAddLoadedTeam.
use super::internal_server_command::InternalServerCommand;

pub struct InternalServerCommandAddLoadedTeam {
    pub game_id: i64,
    pub coach: String,
    pub home_team: Option<bool>,
    /// Account properties (list of strings).
    pub account_properties: Vec<String>,
}

impl InternalServerCommandAddLoadedTeam {
    pub fn new(game_id: i64, coach: String, home_team: Option<bool>, account_properties: Vec<String>) -> Self {
        Self { game_id, coach, home_team, account_properties }
    }

    pub fn get_coach(&self) -> &str {
        &self.coach
    }

    pub fn get_home_team(&self) -> Option<bool> {
        self.home_team
    }

    pub fn get_account_properties(&self) -> &[String] {
        &self.account_properties
    }
}

impl InternalServerCommand for InternalServerCommandAddLoadedTeam {
    fn get_id(&self) -> &'static str {
        "internalServerAddLoadedTeam"
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
        let _ = InternalServerCommandAddLoadedTeam::new(42, "coach".to_string(), Some(true), vec![]);
    }

    #[test]
    fn get_id() {
        let c = InternalServerCommandAddLoadedTeam::new(1, "c".to_string(), None, vec![]);
        assert_eq!(c.get_id(), "internalServerAddLoadedTeam");
    }

    #[test]
    fn get_game_id() {
        let c = InternalServerCommandAddLoadedTeam::new(99, "c".to_string(), None, vec![]);
        assert_eq!(c.get_game_id(), 99);
    }

    #[test]
    fn is_internal() {
        let c = InternalServerCommandAddLoadedTeam::new(1, "c".to_string(), None, vec![]);
        assert!(c.is_internal());
    }

    #[test]
    fn get_coach() {
        let c = InternalServerCommandAddLoadedTeam::new(1, "myCoach".to_string(), None, vec![]);
        assert_eq!(c.get_coach(), "myCoach");
    }
}
