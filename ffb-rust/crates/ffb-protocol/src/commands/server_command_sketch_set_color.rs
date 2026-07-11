use ffb_model::enums::NetCommandId;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandSketchSetColor`.
/// Sets the color of one or more sketches identified by ID.
#[derive(Debug, Clone, Default)]
pub struct ServerCommandSketchSetColor {
    /// Java: base-class `ServerCommand.fCommandNr`.
    pub command_nr: i32,
    /// Java: `coach` — coach who owns the sketches.
    pub coach: String,
    /// Java: `sketchIds` — IDs of sketches to recolor.
    pub sketch_ids: Vec<String>,
    /// Java: `rbg` — packed RGB color value (note: Java field is named `rbg`).
    pub rbg: i32,
}

impl ServerCommandSketchSetColor {
    pub fn new(coach: impl Into<String>, sketch_ids: Vec<String>, rbg: i32) -> Self {
        Self { command_nr: 0, coach: coach.into(), sketch_ids, rbg }
    }
    pub fn get_coach(&self) -> &str { &self.coach }
    pub fn get_sketch_ids(&self) -> &[String] { &self.sketch_ids }
    pub fn get_rbg(&self) -> i32 { self.rbg }

    /// Java: `ServerCommandSketchSetColor.toJsonValue()` — no `commandNr` on
    /// the wire; RGB value is written under the `rgb` key (despite the Java
    /// field being named `rbg`).
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "netCommandId": self.get_id().name(),
            "ids": self.sketch_ids,
            "rgb": self.rbg,
            "coach": self.coach,
        })
    }

    /// Java: `ServerCommandSketchSetColor.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let sketch_ids = json
            .get("ids")
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();
        Self {
            command_nr: 0,
            sketch_ids,
            rbg: json.get("rgb").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            coach: json.get("coach").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        }
    }
}

impl NetCommand for ServerCommandSketchSetColor {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerSketchSetColor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let ids = vec!["s1".to_string(), "s2".to_string()];
        let cmd = ServerCommandSketchSetColor::new("Bob", ids.clone(), 0xFF0000);
        assert_eq!(cmd.get_coach(), "Bob");
        assert_eq!(cmd.get_sketch_ids(), ids.as_slice());
        assert_eq!(cmd.get_rbg(), 0xFF0000);
    }

    #[test]
    fn default_empty() {
        let cmd = ServerCommandSketchSetColor::default();
        assert!(cmd.coach.is_empty());
        assert!(cmd.sketch_ids.is_empty());
        assert_eq!(cmd.rbg, 0);
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ServerCommandSketchSetColor::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ServerCommandSketchSetColor::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ServerCommandSketchSetColor::default());
        assert!(s.contains("ServerCommandSketchSetColor"));
    }

    #[test]
    fn get_id_is_server_sketch_set_color() {
        assert_eq!(ServerCommandSketchSetColor::default().get_id(), NetCommandId::ServerSketchSetColor);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_rgb() {
        let cmd = ServerCommandSketchSetColor::new("Bob", vec!["s1".into()], 0xFF0000);
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "serverSketchSetColor");
        assert_eq!(json["rgb"], 0xFF0000);
        assert_eq!(json["ids"][0], "s1");
        assert_eq!(json["coach"], "Bob");
    }

    #[test]
    fn round_trip_with_data() {
        let cmd = ServerCommandSketchSetColor::new("Carol", vec!["s1".into(), "s2".into()], 42);
        let json = cmd.to_json_value();
        let restored = ServerCommandSketchSetColor::from_json(&json);
        assert_eq!(restored.coach, "Carol");
        assert_eq!(restored.sketch_ids, vec!["s1".to_string(), "s2".to_string()]);
        assert_eq!(restored.rbg, 42);
    }

    #[test]
    fn round_trip_with_default() {
        let cmd = ServerCommandSketchSetColor::default();
        let json = cmd.to_json_value();
        let restored = ServerCommandSketchSetColor::from_json(&json);
        assert!(restored.coach.is_empty());
        assert!(restored.sketch_ids.is_empty());
        assert_eq!(restored.rbg, 0);
    }
}
