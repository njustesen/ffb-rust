use ffb_model::enums::NetCommandId;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandSetPreventSketching`.
/// Instructs the client whether to allow or block sketching.
#[derive(Debug, Clone, Default)]
pub struct ServerCommandSetPreventSketching {
    /// Java: base-class `ServerCommand.fCommandNr`.
    pub command_nr: i32,
    /// Java: `preventSketching` — true = sketching disabled.
    pub prevent_sketching: bool,
    /// Java: `coach`.
    pub coach: String,
}

impl ServerCommandSetPreventSketching {
    pub fn new(coach: impl Into<String>, prevent_sketching: bool) -> Self {
        Self { command_nr: 0, prevent_sketching, coach: coach.into() }
    }
    pub fn is_prevent_sketching(&self) -> bool { self.prevent_sketching }
    pub fn get_coach(&self) -> &str { &self.coach }

    /// Java: `ServerCommandSetPreventSketching.toJsonValue()` — no `commandNr`
    /// on the wire.
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "netCommandId": self.get_id().name(),
            "prevent": self.prevent_sketching,
            "coach": self.coach,
        })
    }

    /// Java: `ServerCommandSetPreventSketching.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            command_nr: 0,
            coach: json.get("coach").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
            prevent_sketching: json.get("prevent").and_then(|v| v.as_bool()).unwrap_or(false),
        }
    }
}

impl NetCommand for ServerCommandSetPreventSketching {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerSetPreventSketching
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flag_stored() {
        let cmd = ServerCommandSetPreventSketching::new("Alice", true);
        assert!(cmd.is_prevent_sketching());
    }

    #[test]
    fn default_allow() {
        let cmd = ServerCommandSetPreventSketching::default();
        assert!(!cmd.prevent_sketching);
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ServerCommandSetPreventSketching::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ServerCommandSetPreventSketching::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ServerCommandSetPreventSketching::default());
        assert!(s.contains("ServerCommandSetPreventSketching"));
    }

    #[test]
    fn get_id_is_server_set_prevent_sketching() {
        assert_eq!(
            ServerCommandSetPreventSketching::new("A", true).get_id(),
            NetCommandId::ServerSetPreventSketching
        );
    }

    #[test]
    fn to_json_value_has_net_command_id_and_fields() {
        let cmd = ServerCommandSetPreventSketching::new("Alice", true);
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "serverSetPreventSketching");
        assert_eq!(json["prevent"], true);
        assert_eq!(json["coach"], "Alice");
    }

    #[test]
    fn round_trip_with_data() {
        let cmd = ServerCommandSetPreventSketching::new("Bob", true);
        let json = cmd.to_json_value();
        let restored = ServerCommandSetPreventSketching::from_json(&json);
        assert_eq!(restored.coach, "Bob");
        assert!(restored.prevent_sketching);
    }

    #[test]
    fn round_trip_with_default() {
        let cmd = ServerCommandSetPreventSketching::default();
        let json = cmd.to_json_value();
        let restored = ServerCommandSetPreventSketching::from_json(&json);
        assert!(restored.coach.is_empty());
        assert!(!restored.prevent_sketching);
    }
}
