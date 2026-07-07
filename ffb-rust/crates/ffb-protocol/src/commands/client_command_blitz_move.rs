use ffb_model::types::FieldCoordinate;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandBlitzMove`.
/// Sent when a player performs a blitz move.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandBlitzMove {
    /// Java: `fActingPlayerId`
    pub acting_player_id: Option<String>,
    /// Java: `fCoordinateFrom`
    pub coordinate_from: Option<FieldCoordinate>,
    /// Java: `fCoordinatesTo`
    pub coordinates_to: Vec<FieldCoordinate>,
}

impl ClientCommandBlitzMove {
    pub fn new() -> Self { Self::default() }

    pub fn with_move(
        acting_player_id: impl Into<String>,
        coordinate_from: FieldCoordinate,
        coordinates_to: Vec<FieldCoordinate>,
    ) -> Self {
        Self {
            acting_player_id: Some(acting_player_id.into()),
            coordinate_from: Some(coordinate_from),
            coordinates_to,
        }
    }

    pub fn get_acting_player_id(&self) -> Option<&str> { self.acting_player_id.as_deref() }
    pub fn get_coordinate_from(&self) -> Option<FieldCoordinate> { self.coordinate_from }
    pub fn get_coordinates_to(&self) -> &[FieldCoordinate] { &self.coordinates_to }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored_correctly() {
        let from = FieldCoordinate::new(1, 1);
        let to = vec![FieldCoordinate::new(2, 2), FieldCoordinate::new(3, 3)];
        let cmd = ClientCommandBlitzMove::with_move("p1", from, to.clone());
        assert_eq!(cmd.get_acting_player_id(), Some("p1"));
        assert_eq!(cmd.get_coordinate_from(), Some(from));
        assert_eq!(cmd.get_coordinates_to().len(), 2);
    }

    #[test]
    fn default_all_none() {
        let cmd = ClientCommandBlitzMove::new();
        assert!(cmd.acting_player_id.is_none());
        assert!(cmd.coordinate_from.is_none());
        assert!(cmd.coordinates_to.is_empty());
    }

    #[test]
    fn coordinates_to_slice_matches_input() {
        let from = FieldCoordinate::new(0, 0);
        let to = vec![FieldCoordinate::new(1, 0)];
        let cmd = ClientCommandBlitzMove::with_move("p2", from, to.clone());
        assert_eq!(cmd.get_coordinates_to(), to.as_slice());
    }
}
