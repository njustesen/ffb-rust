use ffb_model::types::FieldCoordinate;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandSwoop`.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandSwoop {
    /// Java: `fTargetCoordinate`
    pub target_coordinate: Option<FieldCoordinate>,
    /// Java: `fActingPlayerId`
    pub acting_player_id: Option<String>,
}

impl ClientCommandSwoop {
    pub fn new() -> Self { Self::default() }

    pub fn with_players_and_target(
        acting_player_id: impl Into<String>,
        target_coordinate: FieldCoordinate,
    ) -> Self {
        Self {
            acting_player_id: Some(acting_player_id.into()),
            target_coordinate: Some(target_coordinate),
        }
    }

    pub fn get_target_coordinate(&self) -> Option<FieldCoordinate> { self.target_coordinate }
    pub fn get_acting_player_id(&self) -> Option<&str> { self.acting_player_id.as_deref() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let coord = FieldCoordinate::new(7, 3);
        let cmd = ClientCommandSwoop::with_players_and_target("attacker1", coord);
        assert_eq!(cmd.get_acting_player_id(), Some("attacker1"));
        assert_eq!(cmd.get_target_coordinate(), Some(coord));
    }

    #[test]
    fn default_is_empty() {
        let cmd = ClientCommandSwoop::new();
        assert!(cmd.acting_player_id.is_none());
        assert!(cmd.target_coordinate.is_none());
    }
}
