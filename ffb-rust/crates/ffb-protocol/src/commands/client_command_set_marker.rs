use ffb_model::types::FieldCoordinate;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;
use ffb_model::enums::NetCommandId;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandSetMarker`.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandSetMarker {
    /// Java: `fPlayerId`
    pub player_id: Option<String>,
    /// Java: `fCoordinate`
    pub coordinate: Option<FieldCoordinate>,
    /// Java: `fText`
    pub text: Option<String>,
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
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
            entropy: None,
        }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_coordinate(&self) -> Option<FieldCoordinate> { self.coordinate }
    pub fn get_text(&self) -> Option<&str> { self.text.as_deref() }

    /// Java: `ClientCommandSetMarker.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        map.insert(
            "coordinate".to_string(),
            match self.coordinate {
                Some(c) => c.to_json_value(),
                None => serde_json::Value::Null,
            },
        );
        map.insert("playerId".to_string(), match &self.player_id {
            Some(s) => serde_json::json!(s),
            None => serde_json::Value::Null,
        });
        map.insert("text".to_string(), match &self.text {
            Some(s) => serde_json::json!(s),
            None => serde_json::Value::Null,
        });
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandSetMarker.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            coordinate: json.get("coordinate").and_then(FieldCoordinate::from_json),
            player_id: json.get("playerId").and_then(|v| v.as_str()).map(|s| s.to_string()),
            text: json.get("text").and_then(|v| v.as_str()).map(|s| s.to_string()),
            entropy: base.entropy,
        }
    }
}

impl NetCommand for ClientCommandSetMarker {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientSetMarker
    }
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

    #[test]
    fn get_id_is_client_set_marker() {
        assert_eq!(ClientCommandSetMarker::new().get_id(), NetCommandId::ClientSetMarker);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_text() {
        let cmd = ClientCommandSetMarker::with_marker("p1", FieldCoordinate::new(4, 6), "X");
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientSetMarker");
        assert_eq!(json["text"], "X");
        assert_eq!(json["coordinate"], serde_json::json!([4, 6]));
    }

    #[test]
    fn round_trip_populated() {
        let mut cmd = ClientCommandSetMarker::with_marker("p1", FieldCoordinate::new(4, 6), "X");
        cmd.entropy = Some(5);
        let json = cmd.to_json_value();
        let restored = ClientCommandSetMarker::from_json(&json);
        assert_eq!(restored.player_id.as_deref(), Some("p1"));
        assert_eq!(restored.coordinate, Some(FieldCoordinate::new(4, 6)));
        assert_eq!(restored.text.as_deref(), Some("X"));
        assert_eq!(restored.entropy, Some(5));
    }

    #[test]
    fn round_trip_default() {
        let cmd = ClientCommandSetMarker::default();
        let json = cmd.to_json_value();
        let restored = ClientCommandSetMarker::from_json(&json);
        assert_eq!(restored.player_id, None);
        assert_eq!(restored.coordinate, None);
        assert_eq!(restored.text, None);
        assert_eq!(restored.entropy, None);
    }
}
