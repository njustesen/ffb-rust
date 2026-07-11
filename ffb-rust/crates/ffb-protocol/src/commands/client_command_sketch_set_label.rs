use ffb_model::enums::NetCommandId;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandSketchSetLabel`.
/// Java: extends `ClientSketchCommand` (stateless subclass of `ClientCommand`),
/// so the entropy field is still the `ClientCommand` base-class field.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandSketchSetLabel {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `sketchIds`
    pub sketch_ids: Vec<String>,
    /// Java: `label`
    pub label: Option<String>,
}

impl ClientCommandSketchSetLabel {
    pub fn new() -> Self { Self::default() }

    pub fn with_label(sketch_ids: Vec<String>, label: impl Into<String>) -> Self {
        Self { entropy: None, sketch_ids, label: Some(label.into()) }
    }

    pub fn get_sketch_ids(&self) -> &[String] { &self.sketch_ids }
    pub fn get_label(&self) -> Option<&str> { self.label.as_deref() }

    /// Java: `ClientCommandSketchSetLabel.toJsonValue()`. Note the Java class
    /// writes `label` under the `IJsonOption.TEXT` key (`"text"`), not `"label"`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("ids".to_string(), serde_json::json!(self.sketch_ids));
        map.insert("text".to_string(), serde_json::json!(self.label));
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandSketchSetLabel.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            sketch_ids: json.get("ids")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default(),
            label: json.get("text").and_then(|v| v.as_str()).map(|s| s.to_string()),
        }
    }
}

impl NetCommand for ClientCommandSketchSetLabel {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientSketchSetLabel
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let cmd = ClientCommandSketchSetLabel::with_label(
            vec!["s1".to_string()],
            "attack",
        );
        assert_eq!(cmd.get_sketch_ids().len(), 1);
        assert_eq!(cmd.get_label(), Some("attack"));
    }

    #[test]
    fn default_is_empty() {
        let cmd = ClientCommandSketchSetLabel::new();
        assert!(cmd.sketch_ids.is_empty());
        assert!(cmd.label.is_none());
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandSketchSetLabel::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandSketchSetLabel::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandSketchSetLabel::default());
        assert!(s.contains("ClientCommandSketchSetLabel"));
    }

    #[test]
    fn get_id_is_client_sketch_set_label() {
        assert_eq!(ClientCommandSketchSetLabel::new().get_id(), NetCommandId::ClientSketchSetLabel);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_text_key() {
        let cmd = ClientCommandSketchSetLabel::with_label(vec!["s1".to_string()], "attack");
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientSketchSetLabel");
        assert_eq!(json["text"], "attack");
    }

    #[test]
    fn round_trip_with_data() {
        let mut cmd = ClientCommandSketchSetLabel::with_label(
            vec!["s1".to_string(), "s2".to_string()],
            "defend",
        );
        cmd.entropy = Some(6);
        let json = cmd.to_json_value();
        let restored = ClientCommandSketchSetLabel::from_json(&json);
        assert_eq!(restored.sketch_ids, cmd.sketch_ids);
        assert_eq!(restored.label, cmd.label);
        assert_eq!(restored.entropy, cmd.entropy);
    }

    #[test]
    fn round_trip_default() {
        let cmd = ClientCommandSketchSetLabel::default();
        let json = cmd.to_json_value();
        let restored = ClientCommandSketchSetLabel::from_json(&json);
        assert!(restored.sketch_ids.is_empty());
        assert!(restored.label.is_none());
        assert!(restored.entropy.is_none());
    }
}
