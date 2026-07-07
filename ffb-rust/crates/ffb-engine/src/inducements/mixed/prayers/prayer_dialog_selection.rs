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

    #[test]
    fn default_has_no_player() {
        let sel = PrayerDialogSelection::new();
        assert!(sel.get_player_id().is_none());
        assert!(sel.get_team_id().is_none());
    }

    #[test]
    fn with_player_and_team_sets_both() {
        let sel = PrayerDialogSelection::with_player_and_team("p1", "t1");
        assert_eq!(sel.get_player_id(), Some("p1"));
        assert_eq!(sel.get_team_id(), Some("t1"));
    }

    #[test]
    fn with_player_leaves_team_id_none() {
        let sel = PrayerDialogSelection::with_player("player99");
        assert_eq!(sel.get_player_id(), Some("player99"));
        assert!(sel.get_team_id().is_none());
    }

    #[test]
    fn clone_produces_equal_values() {
        let sel = PrayerDialogSelection::with_player_and_team("p2", "t2");
        let cloned = sel.clone();
        assert_eq!(cloned.get_player_id(), sel.get_player_id());
        assert_eq!(cloned.get_team_id(), sel.get_team_id());
    }

    #[test]
    fn default_and_new_are_equivalent() {
        let a = PrayerDialogSelection::new();
        let b = PrayerDialogSelection::default();
        assert_eq!(a.get_player_id(), b.get_player_id());
        assert_eq!(a.get_team_id(), b.get_team_id());
    }

    #[test]
    fn with_player_and_team_team_id_is_correct() {
        let sel = PrayerDialogSelection::with_player_and_team("px", "tx");
        assert_eq!(sel.get_team_id(), Some("tx"));
    }
}
