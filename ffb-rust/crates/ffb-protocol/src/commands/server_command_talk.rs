use ffb_model::enums::NetCommandId;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandTalk`.
/// Delivers chat messages from coaches to all clients.
#[derive(Debug, Clone)]
pub struct ServerCommandTalk {
    /// Java: base-class `ServerCommand.fCommandNr`. Note: this class's
    /// `toJsonValue()`/`initFrom()` never read/write `commandNr`.
    pub command_nr: i32,
    /// Java: `fCoach` — sending coach name.
    pub coach: String,
    /// Java: `fTalks` — list of chat message strings.
    pub talks: Vec<String>,
    /// Java: `mode` — chat mode (`Mode.REGULAR`/`STAFF`/`DEV`); stored as name string.
    pub mode: String,
}

impl ServerCommandTalk {
    pub fn new(coach: impl Into<String>, talks: Vec<String>, mode: impl Into<String>) -> Self {
        Self { command_nr: 0, coach: coach.into(), talks, mode: mode.into() }
    }
    pub fn get_coach(&self) -> &str { &self.coach }
    pub fn get_talks(&self) -> &[String] { &self.talks }
    pub fn get_mode(&self) -> &str { &self.mode }

    /// Java: `ServerCommandTalk.toJsonValue()` — note no `commandNr` key is written.
    pub fn to_json_value(&self) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        map.insert("netCommandId".to_string(), serde_json::json!(self.get_id().name()));
        map.insert("coach".to_string(), serde_json::json!(self.coach));
        map.insert("talks".to_string(), serde_json::json!(self.talks));
        map.insert("talkMode".to_string(), serde_json::json!(self.mode));
        serde_json::Value::Object(map)
    }

    /// Java: `ServerCommandTalk.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let mut mode = "REGULAR".to_string();
        if let Some(admin_mode) = json.get("adminMode").and_then(|v| v.as_bool()) {
            mode = if admin_mode { "STAFF".to_string() } else { "REGULAR".to_string() };
        } else if let Some(talk_mode) = json.get("talkMode").and_then(|v| v.as_str()) {
            mode = talk_mode.to_string();
        }
        let coach = json.get("coach").and_then(|v| v.as_str()).unwrap_or_default().to_string();
        let talks: Vec<String> = json
            .get("talks")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
            .unwrap_or_default();
        Self { command_nr: 0, coach, talks, mode }
    }
}

impl NetCommand for ServerCommandTalk {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerTalk
    }
}

/// Java: `mode` field defaults to `Mode.REGULAR`.
impl Default for ServerCommandTalk {
    fn default() -> Self {
        Self { command_nr: 0, coach: String::new(), talks: Vec::new(), mode: "REGULAR".to_string() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let cmd = ServerCommandTalk::new("Alice", vec!["hi".into()], "REGULAR");
        assert_eq!(cmd.get_coach(), "Alice");
        assert_eq!(cmd.get_talks(), &["hi"]);
        assert_eq!(cmd.get_mode(), "REGULAR");
    }

    #[test]
    fn default_empty() {
        let cmd = ServerCommandTalk::default();
        assert!(cmd.coach.is_empty());
        assert!(cmd.talks.is_empty());
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ServerCommandTalk::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ServerCommandTalk::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ServerCommandTalk::default());
        assert!(s.contains("ServerCommandTalk"));
    }

    #[test]
    fn get_id_is_server_talk() {
        assert_eq!(ServerCommandTalk::default().get_id(), NetCommandId::ServerTalk);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_coach() {
        let cmd = ServerCommandTalk::new("Alice", vec!["hi".into()], "REGULAR");
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "serverTalk");
        assert_eq!(json["coach"], "Alice");
        assert_eq!(json["talkMode"], "REGULAR");
        assert!(json.get("commandNr").is_none());
    }

    #[test]
    fn round_trip_with_talks() {
        let cmd = ServerCommandTalk::new("Bob", vec!["hi".into(), "there".into()], "STAFF");
        let json = cmd.to_json_value();
        let restored = ServerCommandTalk::from_json(&json);
        assert_eq!(restored.coach, "Bob");
        assert_eq!(restored.talks, vec!["hi".to_string(), "there".to_string()]);
        assert_eq!(restored.mode, "STAFF");
    }

    #[test]
    fn round_trip_with_default() {
        let cmd = ServerCommandTalk::default();
        let json = cmd.to_json_value();
        let restored = ServerCommandTalk::from_json(&json);
        assert!(restored.coach.is_empty());
        assert!(restored.talks.is_empty());
        assert_eq!(restored.mode, "REGULAR");
    }
}
