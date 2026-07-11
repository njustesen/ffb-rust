use ffb_model::enums::NetCommandId;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of com.fumbbl.ffb.net.commands.ClientCommandUseApothecaries.
///
/// Java: `fInjuryDescriptions` is a List<InjuryDescription>. InjuryDescription is a complex
/// object — DEFERRED: stored as raw JSON strings until InjuryDescription is fully translated.

#[derive(Debug, Clone, Default)]
pub struct ClientCommandUseApothecaries {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `fInjuryDescriptions` — DEFERRED: InjuryDescription is complex; stored as JSON strings.
    pub injury_description_json: Vec<String>,
}

impl ClientCommandUseApothecaries {
    pub fn new() -> Self {
        Self::default()
    }

    /// Java: `getInjuryDescriptions()` — DEFERRED: returns raw JSON strings for each InjuryDescription.
    pub fn get_injury_description_json(&self) -> &[String] {
        &self.injury_description_json
    }

    /// Java: `ClientCommandUseApothecaries.toJsonValue()` — DEFERRED: each raw JSON string is
    /// parsed back to a `serde_json::Value` so the wire array holds objects (matching
    /// `InjuryDescription::toJsonValue()`), not string-encoded JSON.
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

    /// Java: `ClientCommandUseApothecaries.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        let injury_description_json = json
            .get("injuryDescriptions")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().map(|v| v.to_string()).collect())
            .unwrap_or_default();
        Self {
            entropy: base.entropy,
            injury_description_json,
        }
    }
}

impl NetCommand for ClientCommandUseApothecaries {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientUseApothecaries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_empty_descriptions() {
        let cmd = ClientCommandUseApothecaries::new();
        assert!(cmd.get_injury_description_json().is_empty());
    }

    #[test]
    fn stores_injury_descriptions() {
        let cmd = ClientCommandUseApothecaries {
            entropy: None,
            injury_description_json: vec![
                r#"{"playerId":"p1","apothecaryUsed":true}"#.to_string(),
            ],
        };
        assert_eq!(cmd.get_injury_description_json().len(), 1);
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandUseApothecaries::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandUseApothecaries::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandUseApothecaries::default());
        assert!(s.contains("ClientCommandUseApothecaries"));
    }

    #[test]
    fn get_id_is_client_use_apothecaries() {
        assert_eq!(ClientCommandUseApothecaries::new().get_id(), NetCommandId::ClientUseApothecaries);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_injury_descriptions() {
        let cmd = ClientCommandUseApothecaries {
            entropy: None,
            injury_description_json: vec![r#"{"playerId":"p1"}"#.to_string()],
        };
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientUseApothecaries");
        assert_eq!(json["injuryDescriptions"][0]["playerId"], "p1");
    }

    #[test]
    fn round_trip_with_descriptions_and_entropy() {
        let mut cmd = ClientCommandUseApothecaries {
            entropy: None,
            injury_description_json: vec![r#"{"playerId":"p2","apothecaryUsed":true}"#.to_string()],
        };
        cmd.entropy = Some(9);
        let json = cmd.to_json_value();
        let restored = ClientCommandUseApothecaries::from_json(&json);
        assert_eq!(restored.entropy, Some(9));
        assert_eq!(restored.get_injury_description_json().len(), 1);
        let parsed: serde_json::Value = serde_json::from_str(&restored.injury_description_json[0]).unwrap();
        assert_eq!(parsed["playerId"], "p2");
    }

    #[test]
    fn round_trip_with_empty_descriptions() {
        let cmd = ClientCommandUseApothecaries::new();
        let json = cmd.to_json_value();
        let restored = ClientCommandUseApothecaries::from_json(&json);
        assert!(restored.get_injury_description_json().is_empty());
    }
}
