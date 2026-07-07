use ffb_model::model::ConcedeGameStatus;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandConcedeGame`.
/// Sent when a coach concedes the game.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandConcedeGame {
    /// Java: `fConcedeGameStatus`
    pub concede_game_status: Option<ConcedeGameStatus>,
}

impl ClientCommandConcedeGame {
    pub fn new() -> Self { Self::default() }
    pub fn get_concede_game_status(&self) -> Option<&ConcedeGameStatus> { self.concede_game_status.as_ref() }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_status_none() {
        let cmd = ClientCommandConcedeGame::new();
        assert!(cmd.concede_game_status.is_none());
    }

    #[test]
    fn default_same_as_new() { let _ = ClientCommandConcedeGame::default(); }

    #[test]
    fn stores_concede_status() {
        let cmd = ClientCommandConcedeGame {
            concede_game_status: Some(ConcedeGameStatus::REQUESTED),
        };
        assert!(cmd.get_concede_game_status().is_some());
    }
}
