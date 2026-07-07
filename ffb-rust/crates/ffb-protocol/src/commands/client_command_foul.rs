/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandFoul`.
/// Sent when a player initiates a foul action.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandFoul {
    /// Java: `fActingPlayerId`
    pub acting_player_id: Option<String>,
    /// Java: `fDefenderId`
    pub defender_id: Option<String>,
    /// Java: `usingChainsaw`
    pub using_chainsaw: bool,
}

impl ClientCommandFoul {
    pub fn new() -> Self { Self::default() }

    pub fn with_players(
        acting_player_id: impl Into<String>,
        defender_id: impl Into<String>,
        using_chainsaw: bool,
    ) -> Self {
        Self {
            acting_player_id: Some(acting_player_id.into()),
            defender_id: Some(defender_id.into()),
            using_chainsaw,
        }
    }

    pub fn get_acting_player_id(&self) -> Option<&str> { self.acting_player_id.as_deref() }
    pub fn get_defender_id(&self) -> Option<&str> { self.defender_id.as_deref() }
    pub fn is_using_chainsaw(&self) -> bool { self.using_chainsaw }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored_correctly() {
        let cmd = ClientCommandFoul::with_players("atk", "def", true);
        assert_eq!(cmd.get_acting_player_id(), Some("atk"));
        assert_eq!(cmd.get_defender_id(), Some("def"));
        assert!(cmd.is_using_chainsaw());
    }

    #[test]
    fn default_all_none_and_false() {
        let cmd = ClientCommandFoul::new();
        assert!(cmd.acting_player_id.is_none());
        assert!(cmd.defender_id.is_none());
        assert!(!cmd.using_chainsaw);
    }

    #[test]
    fn no_chainsaw_flag() {
        let cmd = ClientCommandFoul::with_players("a", "b", false);
        assert!(!cmd.is_using_chainsaw());
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandFoul::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandFoul::default().clone();
    }
}
