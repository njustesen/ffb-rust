/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandHandOver`.
/// Sent when a player performs a hand-off to a nearby teammate.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandHandOver {
    /// Java: `fActingPlayerId`
    pub acting_player_id: Option<String>,
    /// Java: `fCatcherId`
    pub catcher_id: Option<String>,
}

impl ClientCommandHandOver {
    pub fn new() -> Self { Self::default() }

    pub fn with_players(
        acting_player_id: impl Into<String>,
        catcher_id: impl Into<String>,
    ) -> Self {
        Self {
            acting_player_id: Some(acting_player_id.into()),
            catcher_id: Some(catcher_id.into()),
        }
    }

    pub fn get_acting_player_id(&self) -> Option<&str> { self.acting_player_id.as_deref() }
    pub fn get_catcher_id(&self) -> Option<&str> { self.catcher_id.as_deref() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored_correctly() {
        let cmd = ClientCommandHandOver::with_players("thrower", "catcher");
        assert_eq!(cmd.get_acting_player_id(), Some("thrower"));
        assert_eq!(cmd.get_catcher_id(), Some("catcher"));
    }

    #[test]
    fn default_both_none() {
        let cmd = ClientCommandHandOver::new();
        assert!(cmd.acting_player_id.is_none());
        assert!(cmd.catcher_id.is_none());
    }
#[test]    fn new_constructor_creates_default() {        let cmd = ClientCommandHandOver::new();        let _ = cmd;    }
}
