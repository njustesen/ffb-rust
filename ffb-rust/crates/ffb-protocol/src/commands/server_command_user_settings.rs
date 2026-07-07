use std::collections::HashMap;
use ffb_model::model::CommonProperty;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandUserSettings`.
/// Delivers persisted user-preference key-value pairs to the client.
#[derive(Debug, Clone, Default)]
pub struct ServerCommandUserSettings {
    /// Java: `fUserSettings` — map from CommonProperty to its stored string value.
    pub user_settings: HashMap<CommonProperty, String>,
}

impl ServerCommandUserSettings {
    pub fn new(user_settings: HashMap<CommonProperty, String>) -> Self { Self { user_settings } }
    pub fn add_user_setting(&mut self, name: CommonProperty, value: impl Into<String>) {
        self.user_settings.insert(name, value.into());
    }
    pub fn get_user_setting_names(&self) -> Vec<CommonProperty> {
        self.user_settings.keys().copied().collect()
    }
    pub fn get_user_setting_value(&self, name: CommonProperty) -> Option<&str> {
        self.user_settings.get(&name).map(String::as_str)
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
}
