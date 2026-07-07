use ffb_model::types::FieldCoordinate;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandPass`.
/// Sent when a player attempts a pass.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandPass {
    /// Java: `fActingPlayerId`
    pub acting_player_id: Option<String>,
    /// Java: `fTargetCoordinate`
    pub target_coordinate: Option<FieldCoordinate>,
}

impl ClientCommandPass {
    pub fn new(acting_player_id: impl Into<String>, target_coordinate: FieldCoordinate) -> Self {
        Self {
            acting_player_id: Some(acting_player_id.into()),
            target_coordinate: Some(target_coordinate),
        }
    }

    pub fn get_acting_player_id(&self) -> Option<&str> { self.acting_player_id.as_deref() }
    pub fn get_target_coordinate(&self) -> Option<FieldCoordinate> { self.target_coordinate }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let coord = FieldCoordinate::new(10, 5);
        let cmd = ClientCommandPass::new("thrower1", coord);
        assert_eq!(cmd.get_acting_player_id(), Some("thrower1"));
        assert_eq!(cmd.get_target_coordinate(), Some(coord));
    }

    #[test]
    fn default_is_empty() {
        let cmd = ClientCommandPass::default();
        assert!(cmd.acting_player_id.is_none());
        assert!(cmd.target_coordinate.is_none());
    }

    #[test]
    fn new_with_coord() {
        let coord = FieldCoordinate::new(0, 0);
        let cmd = ClientCommandPass::new("p1", coord);
        assert!(cmd.get_acting_player_id().is_some());
    }
}
