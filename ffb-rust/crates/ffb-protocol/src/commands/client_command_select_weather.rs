use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;
use ffb_model::enums::NetCommandId;

/// 1:1 translation of ClientCommandSelectWeather (Java fields: modifier, weatherName).
#[derive(Debug, Clone, Default)]
pub struct ClientCommandSelectWeather {
    pub modifier: i32,
    pub weather_name: Option<String>,
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
}

impl ClientCommandSelectWeather {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_fields(modifier: i32, weather_name: impl Into<String>) -> Self {
        Self {
            modifier,
            weather_name: Some(weather_name.into()),
            entropy: None,
        }
    }

    pub fn get_modifier(&self) -> i32 {
        self.modifier
    }

    pub fn get_weather_name(&self) -> Option<&str> {
        self.weather_name.as_deref()
    }

    /// Java: `ClientCommandSelectWeather.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("modifier".to_string(), serde_json::json!(self.modifier));
        map.insert("name".to_string(), match &self.weather_name {
            Some(s) => serde_json::json!(s),
            None => serde_json::Value::Null,
        });
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandSelectWeather.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            modifier: json.get("modifier").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            weather_name: json.get("name").and_then(|v| v.as_str()).map(|s| s.to_string()),
            entropy: base.entropy,
        }
    }
}

impl NetCommand for ClientCommandSelectWeather {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientSelectWeather
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_zero_modifier_and_no_name() {
        let cmd = ClientCommandSelectWeather::new();
        assert_eq!(cmd.get_modifier(), 0);
        assert!(cmd.get_weather_name().is_none());
    }

    #[test]
    fn with_fields_stores_values() {
        let cmd = ClientCommandSelectWeather::with_fields(2, "Nice");
        assert_eq!(cmd.get_modifier(), 2);
        assert_eq!(cmd.get_weather_name(), Some("Nice"));
    }

    #[test]
    fn negative_modifier_stored() {
        let cmd = ClientCommandSelectWeather::with_fields(-1, "Sweltering Heat");
        assert_eq!(cmd.get_modifier(), -1);
        assert_eq!(cmd.get_weather_name(), Some("Sweltering Heat"));
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandSelectWeather::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandSelectWeather::default().clone();
    }

    #[test]
    fn get_id_is_client_select_weather() {
        assert_eq!(ClientCommandSelectWeather::new().get_id(), NetCommandId::ClientSelectWeather);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_modifier() {
        let cmd = ClientCommandSelectWeather::with_fields(3, "Sunny");
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientSelectWeather");
        assert_eq!(json["modifier"], 3);
        assert_eq!(json["name"], "Sunny");
    }

    #[test]
    fn round_trip_populated() {
        let mut cmd = ClientCommandSelectWeather::with_fields(-1, "Sweltering Heat");
        cmd.entropy = Some(11);
        let json = cmd.to_json_value();
        let restored = ClientCommandSelectWeather::from_json(&json);
        assert_eq!(restored.modifier, -1);
        assert_eq!(restored.weather_name.as_deref(), Some("Sweltering Heat"));
        assert_eq!(restored.entropy, Some(11));
    }

    #[test]
    fn round_trip_default() {
        let cmd = ClientCommandSelectWeather::default();
        let json = cmd.to_json_value();
        let restored = ClientCommandSelectWeather::from_json(&json);
        assert_eq!(restored.modifier, 0);
        assert_eq!(restored.weather_name, None);
        assert_eq!(restored.entropy, None);
    }
}
