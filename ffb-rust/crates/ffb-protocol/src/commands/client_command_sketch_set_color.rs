use ffb_model::enums::NetCommandId;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandSketchSetColor`.
/// Java field is named `rbg` (sic — typo for rgb) but we use `rgb` in Rust.
/// Java: extends `ClientSketchCommand` (stateless subclass of `ClientCommand`),
/// so the entropy field is still the `ClientCommand` base-class field.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandSketchSetColor {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `sketchIds`
    pub sketch_ids: Vec<String>,
    /// Java: `rbg` (sic — typo for rgb in Java source)
    pub rgb: i32,
}

impl ClientCommandSketchSetColor {
    pub fn new() -> Self { Self::default() }

    pub fn with_color(sketch_ids: Vec<String>, rgb: i32) -> Self {
        Self { entropy: None, sketch_ids, rgb }
    }

    pub fn get_sketch_ids(&self) -> &[String] { &self.sketch_ids }
    pub fn get_rgb(&self) -> i32 { self.rgb }

    /// Java: `ClientCommandSketchSetColor.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("ids".to_string(), serde_json::json!(self.sketch_ids));
        map.insert("rgb".to_string(), serde_json::json!(self.rgb));
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandSketchSetColor.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            sketch_ids: json.get("ids")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default(),
            rgb: json.get("rgb").and_then(|v| v.as_i64()).map(|v| v as i32).unwrap_or_default(),
        }
    }
}

impl NetCommand for ClientCommandSketchSetColor {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientSketchSetColor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let cmd = ClientCommandSketchSetColor::with_color(
            vec!["s1".to_string(), "s2".to_string()],
            0xFF0000,
        );
        assert_eq!(cmd.get_sketch_ids().len(), 2);
        assert_eq!(cmd.get_rgb(), 0xFF0000);
    }

    #[test]
    fn default_is_empty() {
        let cmd = ClientCommandSketchSetColor::new();
        assert!(cmd.sketch_ids.is_empty());
        assert_eq!(cmd.rgb, 0);
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandSketchSetColor::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandSketchSetColor::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandSketchSetColor::default());
        assert!(s.contains("ClientCommandSketchSetColor"));
    }

    #[test]
    fn get_id_is_client_sketch_set_color() {
        assert_eq!(ClientCommandSketchSetColor::new().get_id(), NetCommandId::ClientSketchSetColor);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_rgb() {
        let cmd = ClientCommandSketchSetColor::with_color(vec!["s1".to_string()], 0xFF0000);
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientSketchSetColor");
        assert_eq!(json["rgb"], 0xFF0000);
    }

    #[test]
    fn round_trip_with_data() {
        let mut cmd = ClientCommandSketchSetColor::with_color(
            vec!["s1".to_string(), "s2".to_string()],
            0x00FF00,
        );
        cmd.entropy = Some(5);
        let json = cmd.to_json_value();
        let restored = ClientCommandSketchSetColor::from_json(&json);
        assert_eq!(restored.sketch_ids, cmd.sketch_ids);
        assert_eq!(restored.rgb, cmd.rgb);
        assert_eq!(restored.entropy, cmd.entropy);
    }

    #[test]
    fn round_trip_default() {
        let cmd = ClientCommandSketchSetColor::default();
        let json = cmd.to_json_value();
        let restored = ClientCommandSketchSetColor::from_json(&json);
        assert!(restored.sketch_ids.is_empty());
        assert_eq!(restored.rgb, 0);
        assert!(restored.entropy.is_none());
    }
}
