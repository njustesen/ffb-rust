use ffb_model::types::FieldCoordinate;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandTeamSetupSave`.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandTeamSetupSave {
    /// Java: `fSetupName`
    pub setup_name: Option<String>,
    /// Java: `fPlayerNumbers`
    pub player_numbers: Vec<i32>,
    /// Java: `fPlayerCoordinates`
    pub player_coordinates: Vec<FieldCoordinate>,
}

impl ClientCommandTeamSetupSave {
    pub fn new() -> Self { Self::default() }

    pub fn with_setup(
        setup_name: impl Into<String>,
        player_numbers: Vec<i32>,
        player_coordinates: Vec<FieldCoordinate>,
    ) -> Self {
        Self {
            setup_name: Some(setup_name.into()),
            player_numbers,
            player_coordinates,
        }
    }

    pub fn get_setup_name(&self) -> Option<&str> { self.setup_name.as_deref() }
    pub fn get_player_numbers(&self) -> &[i32] { &self.player_numbers }
    pub fn get_player_coordinates(&self) -> &[FieldCoordinate] { &self.player_coordinates }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let coords = vec![FieldCoordinate::new(1, 2), FieldCoordinate::new(3, 4)];
        let cmd = ClientCommandTeamSetupSave::with_setup("default", vec![1, 2], coords.clone());
        assert_eq!(cmd.get_setup_name(), Some("default"));
        assert_eq!(cmd.get_player_numbers(), &[1, 2]);
        assert_eq!(cmd.get_player_coordinates().len(), 2);
    }

    #[test]
    fn default_is_empty() {
        let cmd = ClientCommandTeamSetupSave::new();
        assert!(cmd.setup_name.is_none());
        assert!(cmd.player_numbers.is_empty());
        assert!(cmd.player_coordinates.is_empty());
    }
}
