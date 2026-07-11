use ffb_model::enums::NetCommandId;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandRemoveSketches`.
/// Java: extends `ClientSketchCommand` (stateless subclass of `ClientCommand`),
/// so the entropy field is still the `ClientCommand` base-class field.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandRemoveSketches {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `ids`
    pub ids: Vec<String>,
}

impl ClientCommandRemoveSketches {
    pub fn new() -> Self { Self::default() }

    pub fn with_ids(ids: Vec<String>) -> Self {
        Self { entropy: None, ids }
    }

    pub fn get_ids(&self) -> &[String] { &self.ids }

    pub fn add_id(&mut self, id: impl Into<String>) {
        self.ids.push(id.into());
    }

    /// Java: `ClientCommandRemoveSketches.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        if !self.ids.is_empty() {
            map.insert("ids".to_string(), serde_json::json!(self.ids));
        }
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandRemoveSketches.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            ids: json
                .get("ids")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default(),
        }
    }
}

impl NetCommand for ClientCommandRemoveSketches {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientRemoveSketches
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_stored() {
        let cmd = ClientCommandRemoveSketches::with_ids(vec!["id1".to_string(), "id2".to_string()]);
        assert_eq!(cmd.get_ids().len(), 2);
        assert_eq!(cmd.get_ids()[0], "id1");
    }

    #[test]
    fn default_is_empty() {
        let cmd = ClientCommandRemoveSketches::new();
        assert!(cmd.ids.is_empty());
    }

    #[test]
    fn add_id_increments_len() {
        let mut cmd = ClientCommandRemoveSketches::new();
        cmd.add_id("sk-1");
        assert_eq!(cmd.get_ids().len(), 1);
        cmd.add_id("sk-2");
        assert_eq!(cmd.get_ids().len(), 2);
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandRemoveSketches::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandRemoveSketches::default().clone();
    }

    #[test]
    fn get_id_is_client_remove_sketches() {
        assert_eq!(ClientCommandRemoveSketches::new().get_id(), NetCommandId::ClientRemoveSketches);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_ids() {
        let cmd = ClientCommandRemoveSketches::with_ids(vec!["sk-1".to_string()]);
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientRemoveSketches");
        assert_eq!(json["ids"], serde_json::json!(["sk-1"]));
    }

    #[test]
    fn round_trip_with_data() {
        let mut cmd = ClientCommandRemoveSketches::with_ids(vec!["a".to_string(), "b".to_string()]);
        cmd.entropy = Some(3);
        let json = cmd.to_json_value();
        let restored = ClientCommandRemoveSketches::from_json(&json);
        assert_eq!(restored.entropy, Some(3));
        assert_eq!(restored.get_ids(), &["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn round_trip_default() {
        let cmd = ClientCommandRemoveSketches::default();
        let json = cmd.to_json_value();
        let restored = ClientCommandRemoveSketches::from_json(&json);
        assert!(restored.ids.is_empty());
        assert!(restored.entropy.is_none());
        assert!(json.get("ids").is_none());
    }
}
