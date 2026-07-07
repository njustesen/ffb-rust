use ffb_model::types::FieldCoordinate;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandThrowTeamMate`.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandThrowTeamMate {
    /// Java: `fTargetCoordinate`
    pub target_coordinate: Option<FieldCoordinate>,
    /// Java: `fThrownPlayerId`
    pub thrown_player_id: Option<String>,
    /// Java: `fActingPlayerId`
    pub acting_player_id: Option<String>,
    /// Java: `kicked`
    pub kicked: bool,
}

impl ClientCommandThrowTeamMate {
    pub fn new(
        target_coordinate: FieldCoordinate,
        thrown_player_id: impl Into<String>,
        acting_player_id: impl Into<String>,
        kicked: bool,
    ) -> Self {
        Self {
            target_coordinate: Some(target_coordinate),
            thrown_player_id: Some(thrown_player_id.into()),
            acting_player_id: Some(acting_player_id.into()),
            kicked,
        }
    }

    pub fn get_target_coordinate(&self) -> Option<FieldCoordinate> { self.target_coordinate }
    pub fn get_thrown_player_id(&self) -> Option<&str> { self.thrown_player_id.as_deref() }
    pub fn get_acting_player_id(&self) -> Option<&str> { self.acting_player_id.as_deref() }
    pub fn is_kicked(&self) -> bool { self.kicked }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_fields_stored() {
        let coord = FieldCoordinate::new(5, 8);
        let cmd = ClientCommandThrowTeamMate::new(coord, "thrown1", "thrower1", true);
        assert_eq!(cmd.get_target_coordinate(), Some(coord));
        assert_eq!(cmd.get_thrown_player_id(), Some("thrown1"));
        assert_eq!(cmd.get_acting_player_id(), Some("thrower1"));
        assert!(cmd.is_kicked());
    }

    #[test]
    fn default_is_empty() {
        let cmd = ClientCommandThrowTeamMate::default();
        assert!(cmd.target_coordinate.is_none());
        assert!(cmd.thrown_player_id.is_none());
        assert!(!cmd.kicked);
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandThrowTeamMate::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandThrowTeamMate::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandThrowTeamMate::default());
        assert!(s.contains("ClientCommandThrowTeamMate"));
    }
}
