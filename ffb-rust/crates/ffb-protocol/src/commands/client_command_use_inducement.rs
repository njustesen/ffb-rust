use ffb_model::enums::NetCommandId;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandUseInducement`.
/// Sent when a coach activates an inducement during the game.
/// Note: InducementType stored as name string; Card stored as name string (full serialisation not yet ported).
#[derive(Debug, Clone, Default)]
pub struct ClientCommandUseInducement {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `fInducementType` — stored as name string.
    pub inducement_type_name: Option<String>,
    /// Java: `fCard` — stored as card name string.
    pub card_name: Option<String>,
    /// Java: `fPlayerIds`
    pub player_ids: Vec<String>,
}

impl ClientCommandUseInducement {
    pub fn new() -> Self { Self::default() }
    pub fn add_player_id(&mut self, id: impl Into<String>) { self.player_ids.push(id.into()); }
    pub fn get_inducement_type_name(&self) -> Option<&str> { self.inducement_type_name.as_deref() }
    pub fn get_card_name(&self) -> Option<&str> { self.card_name.as_deref() }
    pub fn get_player_ids(&self) -> &[String] { &self.player_ids }

    /// Java: `ClientCommandUseInducement.toJsonValue()` (calls `super.toJsonValue()` first).
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        if let Some(name) = &self.inducement_type_name {
            map.insert("inducementType".to_string(), serde_json::json!(name));
        }
        map.insert("playerIds".to_string(), serde_json::json!(self.player_ids));
        if let Some(card) = &self.card_name {
            map.insert("card".to_string(), serde_json::json!(card));
        }
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandUseInducement.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        let player_ids = json
            .get("playerIds")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();
        Self {
            entropy: base.entropy,
            inducement_type_name: json.get("inducementType").and_then(|v| v.as_str()).map(|s| s.to_string()),
            card_name: json.get("card").and_then(|v| v.as_str()).map(|s| s.to_string()),
            player_ids,
        }
    }
}

impl NetCommand for ClientCommandUseInducement {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientUseInducement
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn player_ids_stored() {
        let mut cmd = ClientCommandUseInducement::new();
        cmd.add_player_id("p1");
        assert_eq!(cmd.get_player_ids(), &["p1"]);
    }
    #[test]
    fn default_all_none() {
        let cmd = ClientCommandUseInducement::new();
        assert!(cmd.inducement_type_name.is_none());
        assert!(cmd.player_ids.is_empty());
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandUseInducement::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandUseInducement::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandUseInducement::default());
        assert!(s.contains("ClientCommandUseInducement"));
    }

    #[test]
    fn get_id_is_client_use_inducement() {
        assert_eq!(ClientCommandUseInducement::new().get_id(), NetCommandId::ClientUseInducement);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_inducement_type() {
        let mut cmd = ClientCommandUseInducement::new();
        cmd.inducement_type_name = Some("Bribe".into());
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientUseInducement");
        assert_eq!(json["inducementType"], "Bribe");
    }

    #[test]
    fn round_trip_with_all_fields_and_entropy() {
        let mut cmd = ClientCommandUseInducement::new();
        cmd.entropy = Some(3);
        cmd.inducement_type_name = Some("Bribe".into());
        cmd.card_name = Some("Chop Block".into());
        cmd.add_player_id("p1");
        let json = cmd.to_json_value();
        let restored = ClientCommandUseInducement::from_json(&json);
        assert_eq!(restored.entropy, Some(3));
        assert_eq!(restored.inducement_type_name.as_deref(), Some("Bribe"));
        assert_eq!(restored.card_name.as_deref(), Some("Chop Block"));
        assert_eq!(restored.player_ids, vec!["p1".to_string()]);
    }

    #[test]
    fn round_trip_with_no_optional_fields() {
        let cmd = ClientCommandUseInducement::new();
        let json = cmd.to_json_value();
        let restored = ClientCommandUseInducement::from_json(&json);
        assert!(restored.inducement_type_name.is_none());
        assert!(restored.card_name.is_none());
        assert!(restored.player_ids.is_empty());
    }
}
