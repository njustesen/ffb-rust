use ffb_model::types::FieldCoordinate;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandMove`.
/// Sent when a player moves through a sequence of squares.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandMove {
    /// Java: `fActingPlayerId`
    pub acting_player_id: Option<String>,
    /// Java: `fCoordinateFrom`
    pub coordinate_from: Option<FieldCoordinate>,
    /// Java: `fCoordinatesTo`
    pub coordinates_to: Vec<FieldCoordinate>,
    /// Java: `ballAndChainRrSetting`
    pub ball_and_chain_rr_setting: Option<String>,
}

impl ClientCommandMove {
    pub fn new(
        acting_player_id: impl Into<String>,
        coordinate_from: FieldCoordinate,
        coordinates_to: Vec<FieldCoordinate>,
        ball_and_chain_rr_setting: Option<String>,
    ) -> Self {
        Self {
            acting_player_id: Some(acting_player_id.into()),
            coordinate_from: Some(coordinate_from),
            coordinates_to,
            ball_and_chain_rr_setting,
        }
    }

    pub fn get_acting_player_id(&self) -> Option<&str> { self.acting_player_id.as_deref() }
    pub fn get_coordinate_from(&self) -> Option<FieldCoordinate> { self.coordinate_from }
    pub fn get_coordinates_to(&self) -> &[FieldCoordinate] { &self.coordinates_to }
    pub fn get_ball_and_chain_rr_setting(&self) -> Option<&str> { self.ball_and_chain_rr_setting.as_deref() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let from = FieldCoordinate::new(3, 4);
        let to = vec![FieldCoordinate::new(4, 4), FieldCoordinate::new(5, 4)];
        let cmd = ClientCommandMove::new("p1", from, to.clone(), None);
        assert_eq!(cmd.get_acting_player_id(), Some("p1"));
        assert_eq!(cmd.get_coordinate_from(), Some(from));
        assert_eq!(cmd.get_coordinates_to().len(), 2);
        assert!(cmd.get_ball_and_chain_rr_setting().is_none());
    }

    #[test]
    fn ball_and_chain_setting() {
        let cmd = ClientCommandMove::new("p1", FieldCoordinate::new(1, 1), vec![], Some("ALWAYS".into()));
        assert_eq!(cmd.get_ball_and_chain_rr_setting(), Some("ALWAYS"));
    }

    #[test]
    fn default_empty() {
        let cmd = ClientCommandMove::default();
        assert!(cmd.acting_player_id.is_none());
        assert!(cmd.coordinates_to.is_empty());
    }
}
