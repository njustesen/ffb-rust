use ffb_model::enums::NetCommandId;
use ffb_model::marking::sort_mode::SortMode;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandUpdatePlayerMarkings`.
/// Sent to update automatic player marking settings.
/// Note: SortMode stored as name string (ffb_model::marking::sort_mode::SortMode available if needed).
#[derive(Debug, Clone, Default)]
pub struct ClientCommandUpdatePlayerMarkings {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `auto`
    pub auto: bool,
    /// Java: `sortMode` — stored as name string.
    pub sort_mode_name: Option<String>,
}

impl ClientCommandUpdatePlayerMarkings {
    pub fn new() -> Self { Self::default() }
    pub fn is_auto(&self) -> bool { self.auto }
    pub fn get_sort_mode_name(&self) -> Option<&str> { self.sort_mode_name.as_deref() }

    /// Java: `ClientCommandUpdatePlayerMarkings.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("useAutoMarkings".to_string(), serde_json::json!(self.auto));
        if let Some(sort_mode_name) = self
            .sort_mode_name
            .as_deref()
            .and_then(SortMode::from_name)
            .map(SortMode::name)
        {
            map.insert("sortMode".to_string(), serde_json::json!(sort_mode_name));
        }
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandUpdatePlayerMarkings.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            auto: json.get("useAutoMarkings").and_then(|v| v.as_bool()).unwrap_or(false),
            sort_mode_name: json.get("sortMode").and_then(|v| v.as_str()).map(|s| s.to_string()),
        }
    }
}

impl NetCommand for ClientCommandUpdatePlayerMarkings {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientUpdatePlayerMarkings
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn auto_flag() {
        let mut cmd = ClientCommandUpdatePlayerMarkings::new();
        cmd.auto = true;
        assert!(cmd.is_auto());
    }
    #[test]
    fn default_false() {
        assert!(!ClientCommandUpdatePlayerMarkings::new().auto);
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandUpdatePlayerMarkings::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandUpdatePlayerMarkings::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandUpdatePlayerMarkings::default());
        assert!(s.contains("ClientCommandUpdatePlayerMarkings"));
    }

    #[test]
    fn get_id_is_client_update_player_markings() {
        assert_eq!(ClientCommandUpdatePlayerMarkings::new().get_id(), NetCommandId::ClientUpdatePlayerMarkings);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_use_auto_markings() {
        let mut cmd = ClientCommandUpdatePlayerMarkings::new();
        cmd.auto = true;
        cmd.sort_mode_name = Some("NONE".to_string());
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientUpdatePlayerMarkings");
        assert_eq!(json["useAutoMarkings"], true);
        assert_eq!(json["sortMode"], "NONE");
    }

    #[test]
    fn round_trip_with_sort_mode_and_entropy() {
        let mut cmd = ClientCommandUpdatePlayerMarkings::new();
        cmd.auto = true;
        cmd.sort_mode_name = Some("DEFAULT".to_string());
        cmd.entropy = Some(14);
        let json = cmd.to_json_value();
        let restored = ClientCommandUpdatePlayerMarkings::from_json(&json);
        assert_eq!(restored.entropy, Some(14));
        assert!(restored.is_auto());
        assert_eq!(restored.get_sort_mode_name(), Some("DEFAULT"));
    }

    #[test]
    fn round_trip_with_no_sort_mode() {
        let cmd = ClientCommandUpdatePlayerMarkings::new();
        let json = cmd.to_json_value();
        assert!(json.get("sortMode").is_none());
        let restored = ClientCommandUpdatePlayerMarkings::from_json(&json);
        assert!(restored.get_sort_mode_name().is_none());
        assert!(!restored.is_auto());
    }
}
