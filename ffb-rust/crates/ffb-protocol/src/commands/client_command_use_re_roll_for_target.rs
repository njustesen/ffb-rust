use ffb_model::enums::NetCommandId;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandUseReRollForTarget`.
/// Extends UseReRoll for a specific block target.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandUseReRollForTarget {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `targetId` (from `ClientCommandUseReRollForTarget`)
    pub target_id: Option<String>,
    /// Java: `fReRolledAction` (inherited from `ClientCommandUseReRoll`)
    pub re_rolled_action: Option<String>,
    /// Java: `fReRollSource` (inherited from `ClientCommandUseReRoll`).
    pub re_roll_source: Option<String>,
}

impl ClientCommandUseReRollForTarget {
    pub fn new() -> Self { Self::default() }
    pub fn get_target_id(&self) -> Option<&str> { self.target_id.as_deref() }
    pub fn get_re_rolled_action(&self) -> Option<&str> { self.re_rolled_action.as_deref() }
    pub fn get_re_roll_source(&self) -> Option<&str> { self.re_roll_source.as_deref() }

    /// Java: `ClientCommandUseReRollForTarget.toJsonValue()` (calls `super.toJsonValue()`,
    /// i.e. `ClientCommandUseReRoll.toJsonValue()`, first).
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        if let Some(action) = &self.re_rolled_action {
            map.insert("reRolledAction".to_string(), serde_json::json!(action));
        }
        if let Some(source) = &self.re_roll_source {
            map.insert("reRollSource".to_string(), serde_json::json!(source));
        }
        if let Some(target_id) = &self.target_id {
            map.insert("playerId".to_string(), serde_json::json!(target_id));
        }
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandUseReRollForTarget.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            re_rolled_action: json.get("reRolledAction").and_then(|v| v.as_str()).map(|s| s.to_string()),
            re_roll_source: json.get("reRollSource").and_then(|v| v.as_str()).map(|s| s.to_string()),
            target_id: json.get("playerId").and_then(|v| v.as_str()).map(|s| s.to_string()),
        }
    }
}

impl NetCommand for ClientCommandUseReRollForTarget {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientUseReRollForTarget
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fields_stored() {
        let mut cmd = ClientCommandUseReRollForTarget::new();
        cmd.target_id = Some("p2".into());
        assert_eq!(cmd.get_target_id(), Some("p2"));
    }
    #[test]
    fn default_none() {
        assert!(ClientCommandUseReRollForTarget::new().target_id.is_none());
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandUseReRollForTarget::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandUseReRollForTarget::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandUseReRollForTarget::default());
        assert!(s.contains("ClientCommandUseReRollForTarget"));
    }

    #[test]
    fn get_id_is_client_use_re_roll_for_target() {
        assert_eq!(ClientCommandUseReRollForTarget::new().get_id(), NetCommandId::ClientUseReRollForTarget);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_player_id() {
        let mut cmd = ClientCommandUseReRollForTarget::new();
        cmd.target_id = Some("p2".into());
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientUseReRollForTarget");
        assert_eq!(json["playerId"], "p2");
    }

    #[test]
    fn round_trip_with_all_fields_and_entropy() {
        let mut cmd = ClientCommandUseReRollForTarget::new();
        cmd.entropy = Some(2);
        cmd.target_id = Some("p3".into());
        cmd.re_rolled_action = Some("BLOCK".into());
        cmd.re_roll_source = Some("TRR".into());
        let json = cmd.to_json_value();
        let restored = ClientCommandUseReRollForTarget::from_json(&json);
        assert_eq!(restored.entropy, Some(2));
        assert_eq!(restored.target_id.as_deref(), Some("p3"));
        assert_eq!(restored.re_rolled_action.as_deref(), Some("BLOCK"));
        assert_eq!(restored.re_roll_source.as_deref(), Some("TRR"));
    }

    #[test]
    fn round_trip_with_no_fields() {
        let cmd = ClientCommandUseReRollForTarget::new();
        let json = cmd.to_json_value();
        let restored = ClientCommandUseReRollForTarget::from_json(&json);
        assert!(restored.target_id.is_none());
        assert!(restored.re_rolled_action.is_none());
        assert!(restored.re_roll_source.is_none());
    }
}
