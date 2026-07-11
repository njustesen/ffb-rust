use ffb_model::marking::player_marker::PlayerMarker;
use ffb_model::enums::NetCommandId;
use crate::commands::server_command::ServerCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandUpdateLocalPlayerMarkers`.
/// Sends the current set of player markers to the client.
#[derive(Debug, Clone, Default)]
pub struct ServerCommandUpdateLocalPlayerMarkers {
    /// Java: base-class `ServerCommand.fCommandNr`.
    pub command_nr: i32,
    /// Java: `markers` — list of player markers to apply.
    pub markers: Vec<PlayerMarker>,
}

impl ServerCommandUpdateLocalPlayerMarkers {
    pub fn new(markers: Vec<PlayerMarker>) -> Self { Self { command_nr: 0, markers } }
    pub fn get_markers(&self) -> &[PlayerMarker] { &self.markers }

    /// Java: `ServerCommandUpdateLocalPlayerMarkers.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ServerCommand { command_nr: self.command_nr };
        let mut map = base.base_json_fields(self.get_id());
        let markers: Vec<serde_json::Value> = self.markers.iter().map(PlayerMarker::to_json_value).collect();
        map.insert("playerMarkerArray".to_string(), serde_json::Value::Array(markers));
        serde_json::Value::Object(map)
    }

    /// Java: `ServerCommandUpdateLocalPlayerMarkers.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ServerCommand::base_from_json(json);
        let markers = json
            .get("playerMarkerArray")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().map(PlayerMarker::from_json).collect())
            .unwrap_or_default();
        Self { command_nr: base.command_nr, markers }
    }
}

impl NetCommand for ServerCommandUpdateLocalPlayerMarkers {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerUpdateLocalPlayerMarkers
    }
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
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ServerCommandUpdateLocalPlayerMarkers::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ServerCommandUpdateLocalPlayerMarkers::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ServerCommandUpdateLocalPlayerMarkers::default());
        assert!(s.contains("ServerCommandUpdateLocalPlayerMarkers"));
    }

    #[test]
    fn get_id_is_server_update_local_player_markers() {
        assert_eq!(
            ServerCommandUpdateLocalPlayerMarkers::default().get_id(),
            NetCommandId::ServerUpdateLocalPlayerMarkers
        );
    }

    #[test]
    fn to_json_value_has_net_command_id_and_markers() {
        let cmd = ServerCommandUpdateLocalPlayerMarkers::new(vec![PlayerMarker::with_player_id("p1")]);
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "serverUpdateLocalPlayerMarkers");
        assert_eq!(json["playerMarkerArray"][0]["playerId"], "p1");
    }

    #[test]
    fn round_trip_with_markers() {
        let mut marker = PlayerMarker::with_player_id("p1");
        marker.set_home_text("H");
        marker.set_away_text("A");
        let mut cmd = ServerCommandUpdateLocalPlayerMarkers::new(vec![marker]);
        cmd.command_nr = 1;
        let json = cmd.to_json_value();
        let restored = ServerCommandUpdateLocalPlayerMarkers::from_json(&json);
        assert_eq!(restored.command_nr, 1);
        assert_eq!(restored.markers.len(), 1);
        assert_eq!(restored.markers[0].get_player_id(), Some("p1"));
        assert_eq!(restored.markers[0].get_home_text(), Some("H"));
        assert_eq!(restored.markers[0].get_away_text(), Some("A"));
    }

    #[test]
    fn round_trip_with_no_markers() {
        let cmd = ServerCommandUpdateLocalPlayerMarkers::default();
        let json = cmd.to_json_value();
        let restored = ServerCommandUpdateLocalPlayerMarkers::from_json(&json);
        assert!(restored.markers.is_empty());
    }
}
