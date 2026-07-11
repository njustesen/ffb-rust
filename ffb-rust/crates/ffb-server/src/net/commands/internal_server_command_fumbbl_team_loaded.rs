/// 1:1 translation of com.fumbbl.ffb.server.net.commands.InternalServerCommandFumbblTeamLoaded.
use super::internal_server_command::InternalServerCommand;

pub struct InternalServerCommandFumbblTeamLoaded {
    pub game_id: i64,
    pub coach: String,
    pub home_team: bool,
    pub account_properties: Vec<String>,
}

impl InternalServerCommandFumbblTeamLoaded {
    pub fn new(game_id: i64, coach: String, home_team: bool, account_properties: Vec<String>) -> Self {
        Self { game_id, coach, home_team, account_properties }
    }

    pub fn get_coach(&self) -> &str {
        &self.coach
    }

    pub fn is_home_team(&self) -> bool {
        self.home_team
    }

    pub fn get_account_properties(&self) -> &[String] {
        &self.account_properties
    }
}

impl InternalServerCommand for InternalServerCommandFumbblTeamLoaded {
    fn get_id(&self) -> &'static str {
        "internalServerFumbblTeamLoaded"
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
        let _ = InternalServerCommandFumbblTeamLoaded::new(1, "c".to_string(), true, vec![]);
    }

    #[test]
    fn get_id() {
        let c = InternalServerCommandFumbblTeamLoaded::new(1, "c".to_string(), true, vec![]);
        assert_eq!(c.get_id(), "internalServerFumbblTeamLoaded");
    }

    #[test]
    fn get_game_id() {
        let c = InternalServerCommandFumbblTeamLoaded::new(9, "c".to_string(), false, vec![]);
        assert_eq!(c.get_game_id(), 9);
    }

    #[test]
    fn is_home_team() {
        let c = InternalServerCommandFumbblTeamLoaded::new(1, "c".to_string(), true, vec![]);
        assert!(c.is_home_team());
    }

    #[test]
    fn is_internal() {
        let c = InternalServerCommandFumbblTeamLoaded::new(1, "c".to_string(), false, vec![]);
        assert!(c.is_internal());
    }
}
