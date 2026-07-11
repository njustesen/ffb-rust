use std::collections::HashMap;
use ffb_model::model::CommonProperty;
use ffb_model::enums::NetCommandId;
use ffb_model::model::factory_type::FactoryContext;
use crate::commands::server_command::ServerCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandUserSettings`.
/// Delivers persisted user-preference key-value pairs to the client.
#[derive(Debug, Clone, Default)]
pub struct ServerCommandUserSettings {
    /// Java: base-class `ServerCommand.fCommandNr`.
    pub command_nr: i32,
    /// Java: `fUserSettings` — map from CommonProperty to its stored string value.
    pub user_settings: HashMap<CommonProperty, String>,
}

impl ServerCommandUserSettings {
    pub fn new(user_settings: HashMap<CommonProperty, String>) -> Self { Self { command_nr: 0, user_settings } }
    pub fn add_user_setting(&mut self, name: CommonProperty, value: impl Into<String>) {
        self.user_settings.insert(name, value.into());
    }
    pub fn get_user_setting_names(&self) -> Vec<CommonProperty> {
        let mut names: Vec<CommonProperty> = self.user_settings.keys().copied().collect();
        names.sort_by_key(|p| p.get_key());
        names
    }
    pub fn get_user_setting_value(&self, name: CommonProperty) -> Option<&str> {
        self.user_settings.get(&name).map(String::as_str)
    }

    /// Java: `ServerCommandUserSettings.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ServerCommand { command_nr: self.command_nr };
        let mut map = base.base_json_fields(self.get_id());
        let names = self.get_user_setting_names();
        let user_setting_names: Vec<&str> = names.iter().map(|n| n.get_key()).collect();
        let user_setting_values: Vec<&str> = names
            .iter()
            .map(|n| self.get_user_setting_value(*n).unwrap_or_default())
            .collect();
        map.insert("userSettingNames".to_string(), serde_json::json!(user_setting_names));
        map.insert("userSettingValues".to_string(), serde_json::json!(user_setting_values));
        serde_json::Value::Object(map)
    }

    /// Java: `ServerCommandUserSettings.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ServerCommand::base_from_json(json);
        let names: Vec<String> = json
            .get("userSettingNames")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
            .unwrap_or_default();
        let values: Vec<String> = json
            .get("userSettingValues")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
            .unwrap_or_default();
        let mut user_settings = HashMap::new();
        for (name, value) in names.iter().zip(values.iter()) {
            if let Some(property) = CommonProperty::for_key(name) {
                user_settings.insert(property, value.clone());
            }
        }
        Self { command_nr: base.command_nr, user_settings }
    }
}

impl NetCommand for ServerCommandUserSettings {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerUserSettings
    }

    fn get_context(&self) -> FactoryContext {
        FactoryContext::APPLICATION
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_retrieve() {
        let mut cmd = ServerCommandUserSettings::default();
        cmd.add_user_setting(CommonProperty::SETTING_SOUND_MODE, "on");
        assert_eq!(cmd.get_user_setting_value(CommonProperty::SETTING_SOUND_MODE), Some("on"));
    }

    #[test]
    fn missing_key_returns_none() {
        let cmd = ServerCommandUserSettings::default();
        assert!(cmd.get_user_setting_value(CommonProperty::SETTING_SOUND_VOLUME).is_none());
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ServerCommandUserSettings::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ServerCommandUserSettings::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ServerCommandUserSettings::default());
        assert!(s.contains("ServerCommandUserSettings"));
    }

    #[test]
    fn get_id_is_server_user_settings() {
        assert_eq!(ServerCommandUserSettings::default().get_id(), NetCommandId::ServerUserSettings);
    }

    #[test]
    fn get_context_is_application() {
        assert_eq!(ServerCommandUserSettings::default().get_context(), FactoryContext::APPLICATION);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_names() {
        let mut cmd = ServerCommandUserSettings::default();
        cmd.add_user_setting(CommonProperty::SETTING_SOUND_MODE, "on");
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "serverUserSettings");
        assert_eq!(json["userSettingNames"][0], "setting.sound.mode");
        assert_eq!(json["userSettingValues"][0], "on");
    }

    #[test]
    fn round_trip_with_settings() {
        let mut cmd = ServerCommandUserSettings::default();
        cmd.command_nr = 5;
        cmd.add_user_setting(CommonProperty::SETTING_SOUND_MODE, "on");
        cmd.add_user_setting(CommonProperty::SETTING_SOUND_VOLUME, "50");
        let json = cmd.to_json_value();
        let restored = ServerCommandUserSettings::from_json(&json);
        assert_eq!(restored.command_nr, 5);
        assert_eq!(restored.get_user_setting_value(CommonProperty::SETTING_SOUND_MODE), Some("on"));
        assert_eq!(restored.get_user_setting_value(CommonProperty::SETTING_SOUND_VOLUME), Some("50"));
    }

    #[test]
    fn round_trip_with_no_settings() {
        let cmd = ServerCommandUserSettings::default();
        let json = cmd.to_json_value();
        let restored = ServerCommandUserSettings::from_json(&json);
        assert!(restored.user_settings.is_empty());
    }
}
