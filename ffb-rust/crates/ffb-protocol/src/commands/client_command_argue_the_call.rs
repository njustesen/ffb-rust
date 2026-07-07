/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandArgueTheCall`.
/// Sent when a coach argues the call for one or more ejected players.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandArgueTheCall {
    /// Java: `fPlayerIds`
    pub player_ids: Vec<String>,
}

impl ClientCommandArgueTheCall {
    pub fn new() -> Self { Self::default() }

    pub fn with_player_id(player_id: impl Into<String>) -> Self {
        Self { player_ids: vec![player_id.into()] }
    }

    pub fn with_player_ids(player_ids: Vec<String>) -> Self {
        Self { player_ids }
    }

    pub fn get_player_ids(&self) -> &[String] { &self.player_ids }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_empty_player_ids() {
        let cmd = ClientCommandArgueTheCall::new();
        assert!(cmd.get_player_ids().is_empty());
    }

    #[test]
    fn with_player_id_stores_id() {
        let cmd = ClientCommandArgueTheCall::with_player_id("p1");
        assert_eq!(cmd.get_player_ids(), &["p1"]);
    }

    #[test]
    fn with_player_ids_stores_all() {
        let cmd = ClientCommandArgueTheCall::with_player_ids(vec!["a".into(), "b".into()]);
        assert_eq!(cmd.get_player_ids().len(), 2);
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandArgueTheCall::default();
        assert!(!format!("{cmd:?}").is_empty());
    }
}
