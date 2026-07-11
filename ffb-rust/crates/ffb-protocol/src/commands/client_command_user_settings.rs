use std::collections::HashMap;
use ffb_model::enums::NetCommandId;
use ffb_model::model::CommonProperty;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandUserSettings`.
/// Sent to store per-user preference settings.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandUserSettings {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `fSettings: Map<CommonProperty, String>`
    pub settings: HashMap<String, String>,
}

impl ClientCommandUserSettings {
    pub fn new() -> Self { Self::default() }
    pub fn set(&mut self, key: CommonProperty, value: impl Into<String>) {
        self.settings.insert(key.get_key().to_string(), value.into());
    }
    pub fn get(&self, key: CommonProperty) -> Option<&str> {
        self.settings.get(key.get_key()).map(|s| s.as_str())
    }

    /// Java: `ClientCommandUserSettings.toJsonValue()` (calls `super.toJsonValue()` first).
    /// Java sorts setting names (`Arrays.sort`) before zipping into parallel
    /// `settingNames`/`settingValues` arrays; the Rust `HashMap` is sorted by key here
    /// for the same deterministic ordering.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        let mut names: Vec<&String> = self.settings.keys().collect();
        names.sort();
        let values: Vec<&str> = names.iter().map(|k| self.settings[*k].as_str()).collect();
        map.insert("settingNames".to_string(), serde_json::json!(names));
        map.insert("settingValues".to_string(), serde_json::json!(values));
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandUserSettings.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        let names: Vec<String> = json
            .get("settingNames")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();
        let values: Vec<String> = json
            .get("settingValues")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();
        let settings = names.into_iter().zip(values).collect();
        Self { entropy: base.entropy, settings }
    }
}

impl NetCommand for ClientCommandUserSettings {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientUserSettings
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_empty() {
        assert!(ClientCommandUserSettings::new().settings.is_empty());
    }

    #[test]
    fn set_and_get_value() {
        let mut cmd = ClientCommandUserSettings::new();
        cmd.set(CommonProperty::SETTING_SOUND_VOLUME, "80");
        assert_eq!(cmd.get(CommonProperty::SETTING_SOUND_VOLUME), Some("80"));
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandUserSettings::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandUserSettings::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandUserSettings::default());
        assert!(s.contains("ClientCommandUserSettings"));
    }

    #[test]
    fn get_id_is_client_user_settings() {
        assert_eq!(ClientCommandUserSettings::new().get_id(), NetCommandId::ClientUserSettings);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_setting_names() {
        let mut cmd = ClientCommandUserSettings::new();
        cmd.set(CommonProperty::SETTING_SOUND_VOLUME, "80");
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientUserSettings");
        assert_eq!(json["settingNames"][0], CommonProperty::SETTING_SOUND_VOLUME.get_key());
        assert_eq!(json["settingValues"][0], "80");
    }

    #[test]
    fn round_trip_with_settings_and_entropy() {
        let mut cmd = ClientCommandUserSettings::new();
        cmd.entropy = Some(4);
        cmd.set(CommonProperty::SETTING_SOUND_VOLUME, "80");
        let json = cmd.to_json_value();
        let restored = ClientCommandUserSettings::from_json(&json);
        assert_eq!(restored.entropy, Some(4));
        assert_eq!(restored.get(CommonProperty::SETTING_SOUND_VOLUME), Some("80"));
    }

    #[test]
    fn round_trip_with_no_settings() {
        let cmd = ClientCommandUserSettings::new();
        let json = cmd.to_json_value();
        let restored = ClientCommandUserSettings::from_json(&json);
        assert!(restored.settings.is_empty());
    }
}
