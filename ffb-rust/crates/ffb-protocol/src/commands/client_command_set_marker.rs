use ffb_model::types::FieldCoordinate;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandSetMarker`.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandSetMarker {
    /// Java: `fPlayerId`
    pub player_id: Option<String>,
    /// Java: `fCoordinate`
    pub coordinate: Option<FieldCoordinate>,
    /// Java: `fText`
    pub text: Option<String>,
}

impl ClientCommandSetMarker {
    pub fn new() -> Self { Self::default() }

    pub fn with_marker(
        player_id: impl Into<String>,
        coordinate: FieldCoordinate,
        text: impl Into<String>,
    ) -> Self {
        Self {
            player_id: Some(player_id.into()),
            coordinate: Some(coordinate),
            text: Some(text.into()),
        }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_coordinate(&self) -> Option<FieldCoordinate> { self.coordinate }
    pub fn get_text(&self) -> Option<&str> { self.text.as_deref() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_marker_stores_all_fields() {
        let coord = FieldCoordinate::new(4, 6);
        let cmd = ClientCommandSetMarker::with_marker("p1", coord, "X");
        assert_eq!(cmd.get_player_id(), Some("p1"));
        assert_eq!(cmd.get_coordinate(), Some(coord));
        assert_eq!(cmd.get_text(), Some("X"));
    }

    #[test]
    fn default_is_empty() {
        let cmd = ClientCommandSetMarker::new();
        assert!(cmd.player_id.is_none());
        assert!(cmd.coordinate.is_none());
        assert!(cmd.text.is_none());
    }

    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandSetMarker::new()).is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandSetMarker::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandSetMarker::default());
        assert!(s.contains("ClientCommandSetMarker"));
    }
}
