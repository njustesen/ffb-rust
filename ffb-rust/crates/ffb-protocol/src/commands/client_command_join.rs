/// 1:1 translation of com.fumbbl.ffb.net.commands.ClientCommandJoin.
use ffb_model::model::ClientMode;

#[derive(Debug, Clone, Default)]
pub struct ClientCommandJoin {
    /// Java: `fCoach`
    pub coach: Option<String>,
    /// Java: `fPassword`
    pub password: Option<String>,
    /// Java: `fGameName`
    pub game_name: Option<String>,
    /// Java: `fTeamId`
    pub team_id: Option<String>,
    /// Java: `fTeamName`
    pub team_name: Option<String>,
    /// Java: `fGameId`
    pub game_id: i64,
    /// Java: `fClientMode`
    pub client_mode: Option<ClientMode>,
}

impl ClientCommandJoin {
    pub fn new() -> Self {
        Self::default()
    }

    /// Java: `getCoach()`
    pub fn get_coach(&self) -> Option<&str> {
        self.coach.as_deref()
    }

    /// Java: `getPassword()`
    pub fn get_password(&self) -> Option<&str> {
        self.password.as_deref()
    }

    /// Java: `getGameName()`
    pub fn get_game_name(&self) -> Option<&str> {
        self.game_name.as_deref()
    }

    /// Java: `getTeamId()`
    pub fn get_team_id(&self) -> Option<&str> {
        self.team_id.as_deref()
    }

    /// Java: `getTeamName()`
    pub fn get_team_name(&self) -> Option<&str> {
        self.team_name.as_deref()
    }

    /// Java: `getGameId()`
    pub fn get_game_id(&self) -> i64 {
        self.game_id
    }

    /// Java: `getClientMode()`
    pub fn get_client_mode(&self) -> Option<&ClientMode> {
        self.client_mode.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_game_id_is_zero() {
        let cmd = ClientCommandJoin::new();
        assert_eq!(cmd.get_game_id(), 0);
    }

    #[test]
    fn stores_coach_and_game_id() {
        let cmd = ClientCommandJoin {
            coach: Some("TestCoach".to_string()),
            game_id: 42,
            ..Default::default()
        };
        assert_eq!(cmd.get_coach(), Some("TestCoach"));
        assert_eq!(cmd.get_game_id(), 42);
    }
}
