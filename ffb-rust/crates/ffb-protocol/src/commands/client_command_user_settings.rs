use std::collections::HashMap;
use ffb_model::model::CommonProperty;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandUserSettings`.
/// Sent to store per-user preference settings.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandUserSettings {
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
}
