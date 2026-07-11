use ffb_model::enums::NetCommandId;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandSketchSetLabel`.
/// Sets the text label of one or more sketches identified by ID.
#[derive(Debug, Clone, Default)]
pub struct ServerCommandSketchSetLabel {
    /// Java: base-class `ServerCommand.fCommandNr`.
    pub command_nr: i32,
    /// Java: `coach` — coach who owns the sketches.
    pub coach: String,
    /// Java: `sketchIds` — IDs of sketches to relabel.
    pub sketch_ids: Vec<String>,
    /// Java: `label` — the new label text.
    pub label: String,
}

impl ServerCommandSketchSetLabel {
    pub fn new(
        coach: impl Into<String>,
        sketch_ids: Vec<String>,
        label: impl Into<String>,
    ) -> Self {
        Self { command_nr: 0, coach: coach.into(), sketch_ids, label: label.into() }
    }
    pub fn get_coach(&self) -> &str { &self.coach }
    pub fn get_sketch_ids(&self) -> &[String] { &self.sketch_ids }
    pub fn get_label(&self) -> &str { &self.label }

    /// Java: `ServerCommandSketchSetLabel.toJsonValue()` — no `commandNr` on
    /// the wire; `label` is written under the `text` key.
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "netCommandId": self.get_id().name(),
            "ids": self.sketch_ids,
            "text": self.label,
            "coach": self.coach,
        })
    }

    /// Java: `ServerCommandSketchSetLabel.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let sketch_ids = json
            .get("ids")
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();
        Self {
            command_nr: 0,
            sketch_ids,
            label: json.get("text").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
            coach: json.get("coach").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        }
    }
}

impl NetCommand for ServerCommandSketchSetLabel {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerSketchSetLabel
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let ids = vec!["s1".to_string()];
        let cmd = ServerCommandSketchSetLabel::new("Carol", ids.clone(), "Arrow");
        assert_eq!(cmd.get_coach(), "Carol");
        assert_eq!(cmd.get_sketch_ids(), ids.as_slice());
        assert_eq!(cmd.get_label(), "Arrow");
    }

    #[test]
    fn default_empty() {
        let cmd = ServerCommandSketchSetLabel::default();
        assert!(cmd.coach.is_empty());
        assert!(cmd.label.is_empty());
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ServerCommandSketchSetLabel::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ServerCommandSketchSetLabel::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ServerCommandSketchSetLabel::default());
        assert!(s.contains("ServerCommandSketchSetLabel"));
    }

    #[test]
    fn get_id_is_server_sketch_set_label() {
        assert_eq!(ServerCommandSketchSetLabel::default().get_id(), NetCommandId::ServerSketchSetLabel);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_text_key() {
        let cmd = ServerCommandSketchSetLabel::new("Carol", vec!["s1".into()], "Arrow");
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "serverSketchSetLabel");
        assert_eq!(json["text"], "Arrow");
        assert_eq!(json["ids"][0], "s1");
        assert_eq!(json["coach"], "Carol");
    }

    #[test]
    fn round_trip_with_data() {
        let cmd = ServerCommandSketchSetLabel::new("Dave", vec!["s1".into(), "s2".into()], "Circle");
        let json = cmd.to_json_value();
        let restored = ServerCommandSketchSetLabel::from_json(&json);
        assert_eq!(restored.coach, "Dave");
        assert_eq!(restored.sketch_ids, vec!["s1".to_string(), "s2".to_string()]);
        assert_eq!(restored.label, "Circle");
    }

    #[test]
    fn round_trip_with_default() {
        let cmd = ServerCommandSketchSetLabel::default();
        let json = cmd.to_json_value();
        let restored = ServerCommandSketchSetLabel::from_json(&json);
        assert!(restored.coach.is_empty());
        assert!(restored.sketch_ids.is_empty());
        assert!(restored.label.is_empty());
    }
}
