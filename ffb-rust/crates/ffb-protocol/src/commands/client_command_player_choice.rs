use ffb_model::model::PlayerChoiceMode;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandPlayerChoice`.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandPlayerChoice {
    /// Java: `fPlayerChoiceMode`
    pub player_choice_mode: Option<PlayerChoiceMode>,
    /// Java: `fPlayerIds`
    pub player_ids: Vec<String>,
}

impl ClientCommandPlayerChoice {
    pub fn new() -> Self { Self::default() }

    pub fn with_mode(player_choice_mode: PlayerChoiceMode) -> Self {
        Self { player_choice_mode: Some(player_choice_mode), player_ids: Vec::new() }
    }

    pub fn get_player_choice_mode(&self) -> Option<PlayerChoiceMode> { self.player_choice_mode }
    pub fn get_player_ids(&self) -> &[String] { &self.player_ids }

    pub fn add_player_id(&mut self, id: impl Into<String>) {
        self.player_ids.push(id.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_stored_and_ids_added() {
        let mut cmd = ClientCommandPlayerChoice::with_mode(PlayerChoiceMode::BLOCK);
        cmd.add_player_id("p1");
        cmd.add_player_id("p2");
        assert_eq!(cmd.get_player_choice_mode(), Some(PlayerChoiceMode::BLOCK));
        assert_eq!(cmd.get_player_ids().len(), 2);
    }

    #[test]
    fn default_is_empty() {
        let cmd = ClientCommandPlayerChoice::new();
        assert!(cmd.player_choice_mode.is_none());
        assert!(cmd.player_ids.is_empty());
    }

    #[test]
    fn add_single_id_len_is_one() {
        let mut cmd = ClientCommandPlayerChoice::new();
        cmd.add_player_id("p99");
        assert_eq!(cmd.get_player_ids().len(), 1);
    }


    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandPlayerChoice::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandPlayerChoice::default().clone();
    }
}
