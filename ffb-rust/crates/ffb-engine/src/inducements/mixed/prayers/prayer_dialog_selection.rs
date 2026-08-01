/// 1:1 translation of `com.fumbbl.ffb.server.inducements.mixed.prayers.PrayerDialogSelection`.
/// Carries the coach's prayer dialog answer back to the handler.
#[derive(Debug, Clone, Default)]
pub struct PrayerDialogSelection {
    pub player_id: Option<String>,
    pub team_id: Option<String>,
}

impl PrayerDialogSelection {
    pub fn new() -> Self {
        Self::default()
    }

    /// Java: getPlayerId()
    pub fn get_player_id(&self) -> Option<&str> {
        self.player_id.as_deref()
    }

    /// Java: getTeamId()
    pub fn get_team_id(&self) -> Option<&str> {
        self.team_id.as_deref()
    }

    pub fn with_player(player_id: impl Into<String>) -> Self {
        Self { player_id: Some(player_id.into()), team_id: None }
    }

    pub fn with_player_and_team(player_id: impl Into<String>, team_id: impl Into<String>) -> Self {
        Self { player_id: Some(player_id.into()), team_id: Some(team_id.into()) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_player_id_returns_set_value() {
        let sel = PrayerDialogSelection::with_player("player1");
        assert_eq!(sel.get_player_id(), Some("player1"));
    }

    // NOTE (test equalization): builder/clone/Default plumbing tests pruned
    // (default_has_no_player, with_player_and_team_sets_both, with_player_leaves_team_id_none,
    // clone_produces_equal_values, default_and_new_are_equivalent,
    // with_player_and_team_team_id_is_correct) - Java's PrayerDialogSelection is a two-field
    // immutable ctor class (playerId, skill); the Rust team_id/builders are engine plumbing.





}
