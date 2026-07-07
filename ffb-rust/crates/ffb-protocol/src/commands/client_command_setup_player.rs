use ffb_model::types::FieldCoordinate;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandSetupPlayer`.
/// Sent when a player is placed during the setup phase.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandSetupPlayer {
    /// Java: `fPlayerId`
    pub player_id: Option<String>,
    /// Java: `fCoordinate`
    pub coordinate: Option<FieldCoordinate>,
}

impl ClientCommandSetupPlayer {
    pub fn new() -> Self { Self::default() }

    pub fn with_placement(
        player_id: impl Into<String>,
        coordinate: FieldCoordinate,
    ) -> Self {
        Self {
            player_id: Some(player_id.into()),
            coordinate: Some(coordinate),
        }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_coordinate(&self) -> Option<FieldCoordinate> { self.coordinate }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored_correctly() {
        let coord = FieldCoordinate::new(5, 5);
        let cmd = ClientCommandSetupPlayer::with_placement("p1", coord);
        assert_eq!(cmd.get_player_id(), Some("p1"));
        assert_eq!(cmd.get_coordinate(), Some(coord));
    }

    #[test]
    fn default_both_none() {
        let cmd = ClientCommandSetupPlayer::new();
        assert!(cmd.player_id.is_none());
        assert!(cmd.coordinate.is_none());
    }

    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandSetupPlayer::new()).is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandSetupPlayer::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandSetupPlayer::default());
        assert!(s.contains("ClientCommandSetupPlayer"));
    }
}
