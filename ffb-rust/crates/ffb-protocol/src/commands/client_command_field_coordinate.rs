/// 1:1 translation of ClientCommandFieldCoordinate (Java field: fieldCoordinate).
use ffb_model::enums::NetCommandId;
use ffb_model::types::FieldCoordinate;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

#[derive(Debug, Clone, Default)]
pub struct ClientCommandFieldCoordinate {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    pub field_coordinate: Option<FieldCoordinate>,
}

impl ClientCommandFieldCoordinate {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_coordinate(c: FieldCoordinate) -> Self {
        Self { entropy: None, field_coordinate: Some(c) }
    }

    pub fn get_field_coordinate(&self) -> Option<FieldCoordinate> {
        self.field_coordinate
    }

    /// Java: `ClientCommandFieldCoordinate.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        if let Some(fc) = self.field_coordinate {
            map.insert("fieldCoordinate".to_string(), fc.to_json_value());
        }
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandFieldCoordinate.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            field_coordinate: json.get("fieldCoordinate").and_then(FieldCoordinate::from_json),
        }
    }
}

impl NetCommand for ClientCommandFieldCoordinate {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientFieldCoordinate
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::types::FieldCoordinate;

    #[test]
    fn default_has_no_coordinate() {
        let cmd = ClientCommandFieldCoordinate::new();
        assert!(cmd.get_field_coordinate().is_none());
    }

    #[test]
    fn with_coordinate_stores_value() {
        let coord = FieldCoordinate::new(3, 5);
        let cmd = ClientCommandFieldCoordinate::with_coordinate(coord);
        assert_eq!(cmd.get_field_coordinate(), Some(FieldCoordinate::new(3, 5)));
    }

    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandFieldCoordinate::new()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandFieldCoordinate::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandFieldCoordinate::default());
        assert!(s.contains("ClientCommandFieldCoordinate"));
    }

    #[test]
    fn get_id_is_client_field_coordinate() {
        assert_eq!(ClientCommandFieldCoordinate::new().get_id(), NetCommandId::ClientFieldCoordinate);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_coordinate() {
        let cmd = ClientCommandFieldCoordinate::with_coordinate(FieldCoordinate::new(4, 8));
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientFieldCoordinate");
        assert_eq!(json["fieldCoordinate"], serde_json::json!([4, 8]));
    }

    #[test]
    fn round_trip_with_coordinate_and_entropy() {
        let mut cmd = ClientCommandFieldCoordinate::with_coordinate(FieldCoordinate::new(1, 2));
        cmd.entropy = Some(11);
        let json = cmd.to_json_value();
        let restored = ClientCommandFieldCoordinate::from_json(&json);
        assert_eq!(restored.entropy, Some(11));
        assert_eq!(restored.field_coordinate, Some(FieldCoordinate::new(1, 2)));
    }

    #[test]
    fn round_trip_with_no_coordinate() {
        let cmd = ClientCommandFieldCoordinate::new();
        let json = cmd.to_json_value();
        let restored = ClientCommandFieldCoordinate::from_json(&json);
        assert!(restored.field_coordinate.is_none());
    }
}
