use ffb_model::enums::NetCommandId;
use ffb_model::types::FieldCoordinate;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandSketchAddCoordinate`.
/// Java: extends `ClientSketchCommand` (a stateless subclass of `ClientCommand`
/// adding only `requiresControl()`), so the entropy field is still the
/// `ClientCommand` base-class field.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandSketchAddCoordinate {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `sketchId`
    pub sketch_id: Option<String>,
    /// Java: `coordinate`
    pub coordinate: Option<FieldCoordinate>,
}

impl ClientCommandSketchAddCoordinate {
    pub fn new() -> Self { Self::default() }

    pub fn with_sketch(sketch_id: impl Into<String>, coordinate: FieldCoordinate) -> Self {
        Self { entropy: None, sketch_id: Some(sketch_id.into()), coordinate: Some(coordinate) }
    }

    pub fn get_sketch_id(&self) -> Option<&str> { self.sketch_id.as_deref() }
    pub fn get_coordinate(&self) -> Option<FieldCoordinate> { self.coordinate }

    /// Java: `ClientCommandSketchAddCoordinate.toJsonValue()`. Note the Java
    /// class writes `sketchId` under the `IJsonOption.ID` key (`"id"`), not `"sketchId"`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("id".to_string(), serde_json::json!(self.sketch_id));
        if let Some(coordinate) = self.coordinate {
            map.insert("coordinate".to_string(), coordinate.to_json_value());
        }
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandSketchAddCoordinate.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            sketch_id: json.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()),
            coordinate: json.get("coordinate").and_then(FieldCoordinate::from_json),
        }
    }
}

impl NetCommand for ClientCommandSketchAddCoordinate {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientSketchAddCoordinate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let coord = FieldCoordinate::new(2, 9);
        let cmd = ClientCommandSketchAddCoordinate::with_sketch("sketch1", coord);
        assert_eq!(cmd.get_sketch_id(), Some("sketch1"));
        assert_eq!(cmd.get_coordinate(), Some(coord));
    }

    #[test]
    fn default_is_empty() {
        let cmd = ClientCommandSketchAddCoordinate::new();
        assert!(cmd.sketch_id.is_none());
        assert!(cmd.coordinate.is_none());
    }

    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandSketchAddCoordinate::new()).is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandSketchAddCoordinate::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandSketchAddCoordinate::default());
        assert!(s.contains("ClientCommandSketchAddCoordinate"));
    }

    #[test]
    fn get_id_is_client_sketch_add_coordinate() {
        assert_eq!(ClientCommandSketchAddCoordinate::new().get_id(), NetCommandId::ClientSketchAddCoordinate);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_id_key() {
        let cmd = ClientCommandSketchAddCoordinate::with_sketch("sketch1", FieldCoordinate::new(2, 9));
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientSketchAddCoordinate");
        assert_eq!(json["id"], "sketch1");
    }

    #[test]
    fn round_trip_with_data() {
        let mut cmd = ClientCommandSketchAddCoordinate::with_sketch("sketch2", FieldCoordinate::new(4, 4));
        cmd.entropy = Some(11);
        let json = cmd.to_json_value();
        let restored = ClientCommandSketchAddCoordinate::from_json(&json);
        assert_eq!(restored.sketch_id, cmd.sketch_id);
        assert_eq!(restored.coordinate, cmd.coordinate);
        assert_eq!(restored.entropy, cmd.entropy);
    }

    #[test]
    fn round_trip_default() {
        let cmd = ClientCommandSketchAddCoordinate::default();
        let json = cmd.to_json_value();
        let restored = ClientCommandSketchAddCoordinate::from_json(&json);
        assert!(restored.sketch_id.is_none());
        assert!(restored.coordinate.is_none());
        assert!(restored.entropy.is_none());
    }
}
