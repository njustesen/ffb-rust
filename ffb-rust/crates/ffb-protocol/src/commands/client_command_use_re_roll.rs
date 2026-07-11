use ffb_model::enums::NetCommandId;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandUseReRoll`.
/// Sent to consume a team re-roll or skill re-roll.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandUseReRoll {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `fReRolledAction` — name of the action being re-rolled.
    pub re_rolled_action: Option<String>,
    /// Java: `fReRollSource` — name of the re-roll source (TRR, PRO, etc.).
    pub re_roll_source: Option<String>,
}

impl ClientCommandUseReRoll {
    pub fn new(re_rolled_action: impl Into<String>, re_roll_source: impl Into<String>) -> Self {
        Self {
            entropy: None,
            re_rolled_action: Some(re_rolled_action.into()),
            re_roll_source: Some(re_roll_source.into()),
        }
    }

    pub fn get_re_rolled_action(&self) -> Option<&str> { self.re_rolled_action.as_deref() }
    pub fn get_re_roll_source(&self) -> Option<&str> { self.re_roll_source.as_deref() }

    /// Java: `ClientCommandUseReRoll.toJsonValue()` (calls `super.toJsonValue()` first).
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        if let Some(action) = &self.re_rolled_action {
            map.insert("reRolledAction".to_string(), serde_json::json!(action));
        }
        if let Some(source) = &self.re_roll_source {
            map.insert("reRollSource".to_string(), serde_json::json!(source));
        }
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandUseReRoll.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            re_rolled_action: json.get("reRolledAction").and_then(|v| v.as_str()).map(|s| s.to_string()),
            re_roll_source: json.get("reRollSource").and_then(|v| v.as_str()).map(|s| s.to_string()),
        }
    }
}

impl NetCommand for ClientCommandUseReRoll {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientUseReRoll
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let cmd = ClientCommandUseReRoll::new("DODGE", "TRR");
        assert_eq!(cmd.get_re_rolled_action(), Some("DODGE"));
        assert_eq!(cmd.get_re_roll_source(), Some("TRR"));
    }

    #[test]
    fn default_is_empty() {
        let cmd = ClientCommandUseReRoll::default();
        assert!(cmd.re_rolled_action.is_none());
        assert!(cmd.re_roll_source.is_none());
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandUseReRoll::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandUseReRoll::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandUseReRoll::default());
        assert!(s.contains("ClientCommandUseReRoll"));
    }

    #[test]
    fn get_id_is_client_use_re_roll() {
        assert_eq!(ClientCommandUseReRoll::default().get_id(), NetCommandId::ClientUseReRoll);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_re_rolled_action() {
        let cmd = ClientCommandUseReRoll::new("DODGE", "TRR");
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientUseReRoll");
        assert_eq!(json["reRolledAction"], "DODGE");
        assert_eq!(json["reRollSource"], "TRR");
    }

    #[test]
    fn round_trip_with_fields_and_entropy() {
        let mut cmd = ClientCommandUseReRoll::new("BLOCK", "PRO");
        cmd.entropy = Some(1);
        let json = cmd.to_json_value();
        let restored = ClientCommandUseReRoll::from_json(&json);
        assert_eq!(restored.entropy, Some(1));
        assert_eq!(restored.re_rolled_action.as_deref(), Some("BLOCK"));
        assert_eq!(restored.re_roll_source.as_deref(), Some("PRO"));
    }

    #[test]
    fn round_trip_with_no_fields() {
        let cmd = ClientCommandUseReRoll::default();
        let json = cmd.to_json_value();
        let restored = ClientCommandUseReRoll::from_json(&json);
        assert!(restored.re_rolled_action.is_none());
        assert!(restored.re_roll_source.is_none());
    }
}
