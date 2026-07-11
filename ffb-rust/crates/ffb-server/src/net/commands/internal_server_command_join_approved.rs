/// 1:1 translation of com.fumbbl.ffb.server.net.commands.InternalServerCommandJoinApproved.
/// ClientMode stored as String; full enum not yet imported.
use super::internal_server_command::InternalServerCommand;

pub struct InternalServerCommandJoinApproved {
    pub game_id: i64,
    pub game_name: String,
    pub coach: String,
    pub team_id: String,
    /// ClientMode stored as String; full enum not yet available.
    pub client_mode: String,
    pub account_properties: Vec<String>,
}

impl InternalServerCommandJoinApproved {
    pub fn new(
        game_id: i64,
        game_name: String,
        coach: String,
        team_id: String,
        client_mode: String,
        account_properties: Vec<String>,
    ) -> Self {
        Self { game_id, game_name, coach, team_id, client_mode, account_properties }
    }

    pub fn get_coach(&self) -> &str {
        &self.coach
    }

    pub fn get_game_name(&self) -> &str {
        &self.game_name
    }

    pub fn get_client_mode(&self) -> &str {
        &self.client_mode
    }

    pub fn get_team_id(&self) -> &str {
        &self.team_id
    }

    pub fn get_account_properties(&self) -> &[String] {
        &self.account_properties
    }
}

impl InternalServerCommand for InternalServerCommandJoinApproved {
    fn get_id(&self) -> &'static str {
        "internalServerJoinApproved"
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
        let _ = InternalServerCommandJoinApproved::new(
            1, "game".to_string(), "coach".to_string(), "team1".to_string(), "PLAYER".to_string(), vec![]
        );
    }

    #[test]
    fn get_id() {
        let c = InternalServerCommandJoinApproved::new(
            1, "g".to_string(), "c".to_string(), "t".to_string(), "m".to_string(), vec![]
        );
        assert_eq!(c.get_id(), "internalServerJoinApproved");
    }

    #[test]
    fn get_game_id() {
        let c = InternalServerCommandJoinApproved::new(
            5, "g".to_string(), "c".to_string(), "t".to_string(), "m".to_string(), vec![]
        );
        assert_eq!(c.get_game_id(), 5);
    }

    #[test]
    fn is_internal() {
        let c = InternalServerCommandJoinApproved::new(
            1, "g".to_string(), "c".to_string(), "t".to_string(), "m".to_string(), vec![]
        );
        assert!(c.is_internal());
    }

    #[test]
    fn get_team_id() {
        let c = InternalServerCommandJoinApproved::new(
            1, "g".to_string(), "c".to_string(), "myTeam".to_string(), "m".to_string(), vec![]
        );
        assert_eq!(c.get_team_id(), "myTeam");
    }
}
