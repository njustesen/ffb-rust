use ffb_model::marking::player_marker::PlayerMarker;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandUpdateLocalPlayerMarkers`.
/// Sends the current set of player markers to the client.
#[derive(Debug, Clone, Default)]
pub struct ServerCommandUpdateLocalPlayerMarkers {
    /// Java: `markers` — list of player markers to apply.
    pub markers: Vec<PlayerMarker>,
}

impl ServerCommandUpdateLocalPlayerMarkers {
    pub fn new(markers: Vec<PlayerMarker>) -> Self { Self { markers } }
    pub fn get_markers(&self) -> &[PlayerMarker] { &self.markers }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let cmd = ServerCommandUpdateLocalPlayerMarkers::new(vec![PlayerMarker::default()]);
        assert_eq!(cmd.get_markers().len(), 1);
    }

    #[test]
    fn default_empty() {
        let cmd = ServerCommandUpdateLocalPlayerMarkers::default();
        assert!(cmd.markers.is_empty());
    }
}
