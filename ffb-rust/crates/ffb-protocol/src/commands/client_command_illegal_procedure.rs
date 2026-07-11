/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandIllegalProcedure`.
/// Sent when a coach invokes the Illegal Procedure ruling (no payload).
use ffb_model::enums::NetCommandId;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

#[derive(Debug, Clone, Default)]
pub struct ClientCommandIllegalProcedure {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
}

impl ClientCommandIllegalProcedure {
    pub fn new() -> Self { Self::default() }

    /// Java: `ClientCommandIllegalProcedure` has no `toJsonValue()` override — only inherits
    /// the base-class fields (there is no `initFrom` override that adds fields either, only
    /// the base-class dispatch call).
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        serde_json::Value::Object(base.base_json_fields(self.get_id()))
    }

    /// Java: `ClientCommandIllegalProcedure.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self { entropy: base.entropy }
    }
}

impl NetCommand for ClientCommandIllegalProcedure {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientIllegalProcedure
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_construct() {
        let _ = ClientCommandIllegalProcedure::new();
    }

    #[test]
    fn default_same_as_new() {
        let _ = ClientCommandIllegalProcedure::default();
    }

    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandIllegalProcedure::new()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandIllegalProcedure::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandIllegalProcedure::default());
        assert!(s.contains("ClientCommandIllegalProcedure"));
    }

    #[test]
    fn get_id_is_client_illegal_procedure() {
        assert_eq!(ClientCommandIllegalProcedure::new().get_id(), NetCommandId::ClientIllegalProcedure);
    }

    #[test]
    fn to_json_value_has_net_command_id() {
        let cmd = ClientCommandIllegalProcedure::new();
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientIllegalProcedure");
    }

    #[test]
    fn round_trip_with_entropy() {
        let cmd = ClientCommandIllegalProcedure { entropy: Some(1) };
        let json = cmd.to_json_value();
        let restored = ClientCommandIllegalProcedure::from_json(&json);
        assert_eq!(restored.entropy, Some(1));
    }

    #[test]
    fn round_trip_default() {
        let cmd = ClientCommandIllegalProcedure::default();
        let json = cmd.to_json_value();
        let restored = ClientCommandIllegalProcedure::from_json(&json);
        assert!(restored.entropy.is_none());
    }
}
