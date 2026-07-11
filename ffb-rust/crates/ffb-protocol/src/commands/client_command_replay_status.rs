use ffb_model::enums::NetCommandId;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandReplayStatus`.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandReplayStatus {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `commandNr`
    pub command_nr: i32,
    /// Java: `speed`
    pub speed: i32,
    /// Java: `running`
    pub running: bool,
    /// Java: `forward`
    pub forward: bool,
    /// Java: `skip`
    pub skip: bool,
}

impl ClientCommandReplayStatus {
    pub fn new() -> Self { Self::default() }

    pub fn with_params(command_nr: i32, speed: i32, running: bool, forward: bool, skip: bool) -> Self {
        Self { entropy: None, command_nr, speed, running, forward, skip }
    }

    pub fn get_command_nr(&self) -> i32 { self.command_nr }
    pub fn get_speed(&self) -> i32 { self.speed }
    pub fn is_running(&self) -> bool { self.running }
    pub fn is_forward(&self) -> bool { self.forward }
    pub fn is_skip(&self) -> bool { self.skip }

    /// Java: `ClientCommandReplayStatus.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("commandNr".to_string(), serde_json::json!(self.command_nr));
        map.insert("running".to_string(), serde_json::json!(self.running));
        map.insert("forward".to_string(), serde_json::json!(self.forward));
        map.insert("speed".to_string(), serde_json::json!(self.speed));
        map.insert("skip".to_string(), serde_json::json!(self.skip));
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandReplayStatus.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            forward: json.get("forward").and_then(|v| v.as_bool()).unwrap_or(false),
            running: json.get("running").and_then(|v| v.as_bool()).unwrap_or(false),
            command_nr: json.get("commandNr").and_then(|v| v.as_i64()).map(|v| v as i32).unwrap_or(0),
            speed: json.get("speed").and_then(|v| v.as_i64()).map(|v| v as i32).unwrap_or(0),
            skip: json.get("skip").and_then(|v| v.as_bool()).unwrap_or(false),
        }
    }
}

impl NetCommand for ClientCommandReplayStatus {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientReplayStatus
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let cmd = ClientCommandReplayStatus::with_params(55, 2, true, true, false);
        assert_eq!(cmd.get_command_nr(), 55);
        assert_eq!(cmd.get_speed(), 2);
        assert!(cmd.is_running());
        assert!(cmd.is_forward());
        assert!(!cmd.is_skip());
    }

    #[test]
    fn default_is_zeroed() {
        let cmd = ClientCommandReplayStatus::new();
        assert_eq!(cmd.command_nr, 0);
        assert!(!cmd.running);
        assert!(!cmd.skip);
    }

    #[test]
    fn skip_can_be_set() {
        let cmd = ClientCommandReplayStatus::with_params(0, 1, false, false, true);
        assert!(cmd.is_skip());
        assert!(!cmd.is_running());
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandReplayStatus::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandReplayStatus::default().clone();
    }

    #[test]
    fn get_id_is_client_replay_status() {
        assert_eq!(ClientCommandReplayStatus::new().get_id(), NetCommandId::ClientReplayStatus);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_fields() {
        let cmd = ClientCommandReplayStatus::with_params(10, 3, true, false, true);
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientReplayStatus");
        assert_eq!(json["commandNr"], 10);
        assert_eq!(json["speed"], 3);
        assert_eq!(json["running"], true);
        assert_eq!(json["forward"], false);
        assert_eq!(json["skip"], true);
    }

    #[test]
    fn round_trip_with_data() {
        let mut cmd = ClientCommandReplayStatus::with_params(20, 5, true, true, false);
        cmd.entropy = Some(1);
        let json = cmd.to_json_value();
        let restored = ClientCommandReplayStatus::from_json(&json);
        assert_eq!(restored.entropy, Some(1));
        assert_eq!(restored.get_command_nr(), 20);
        assert_eq!(restored.get_speed(), 5);
        assert!(restored.is_running());
        assert!(restored.is_forward());
        assert!(!restored.is_skip());
    }

    #[test]
    fn round_trip_default() {
        let cmd = ClientCommandReplayStatus::default();
        let json = cmd.to_json_value();
        let restored = ClientCommandReplayStatus::from_json(&json);
        assert_eq!(restored.command_nr, 0);
        assert_eq!(restored.speed, 0);
        assert!(!restored.running);
        assert!(!restored.forward);
        assert!(!restored.skip);
        assert!(restored.entropy.is_none());
    }
}
