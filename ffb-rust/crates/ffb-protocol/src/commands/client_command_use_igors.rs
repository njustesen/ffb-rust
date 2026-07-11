use ffb_model::enums::NetCommandId;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandUseIgors`.
/// Sent when Necromantic team uses Igors for injury recovery.
/// Note: InjuryDescription serialised as raw JSON strings; full type not yet ported.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandUseIgors {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `injuryDescriptions` — simplified to JSON strings pending InjuryDescription port.
    pub injury_description_json: Vec<String>,
}

impl ClientCommandUseIgors {
    pub fn new() -> Self { Self::default() }
    pub fn get_injury_descriptions(&self) -> &[String] { &self.injury_description_json }

    /// Java: `ClientCommandUseIgors.toJsonValue()` (calls `super.toJsonValue()` first).
    /// Each stored raw-JSON string is re-parsed into a `JsonValue` and collected into
    /// the `injuryDescriptions` array (Java: `InjuryDescription::toJsonValue` per element).
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        let array: Vec<serde_json::Value> = self
            .injury_description_json
            .iter()
            .map(|s| serde_json::from_str(s).unwrap_or(serde_json::Value::Null))
            .collect();
        map.insert("injuryDescriptions".to_string(), serde_json::Value::Array(array));
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandUseIgors.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        let injury_description_json = json
            .get("injuryDescriptions")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().map(|v| v.to_string()).collect())
            .unwrap_or_default();
        Self { entropy: base.entropy, injury_description_json }
    }
}

impl NetCommand for ClientCommandUseIgors {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientUseIgors
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_empty() {
        assert!(ClientCommandUseIgors::new().injury_description_json.is_empty());
    }

    #[test]
    fn getter_returns_slice() {
        let mut cmd = ClientCommandUseIgors::new();
        cmd.injury_description_json.push("{}".into());
        assert_eq!(cmd.get_injury_descriptions().len(), 1);
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandUseIgors::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandUseIgors::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandUseIgors::default());
        assert!(s.contains("ClientCommandUseIgors"));
    }

    #[test]
    fn get_id_is_client_use_igors() {
        assert_eq!(ClientCommandUseIgors::new().get_id(), NetCommandId::ClientUseIgors);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_injury_descriptions() {
        let mut cmd = ClientCommandUseIgors::new();
        cmd.injury_description_json.push(r#"{"playerId":"p1"}"#.into());
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientUseIgors");
        assert_eq!(json["injuryDescriptions"][0]["playerId"], "p1");
    }

    #[test]
    fn round_trip_with_descriptions_and_entropy() {
        let mut cmd = ClientCommandUseIgors::new();
        cmd.entropy = Some(7);
        cmd.injury_description_json.push(r#"{"playerId":"p1"}"#.into());
        let json = cmd.to_json_value();
        let restored = ClientCommandUseIgors::from_json(&json);
        assert_eq!(restored.entropy, Some(7));
        assert_eq!(restored.injury_description_json.len(), 1);
        let parsed: serde_json::Value = serde_json::from_str(&restored.injury_description_json[0]).unwrap();
        assert_eq!(parsed["playerId"], "p1");
    }

    #[test]
    fn round_trip_with_no_descriptions() {
        let cmd = ClientCommandUseIgors::new();
        let json = cmd.to_json_value();
        let restored = ClientCommandUseIgors::from_json(&json);
        assert!(restored.injury_description_json.is_empty());
    }
}
