use ffb_model::enums::NetCommandId;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandUseProReRollForBlock`.
/// Sent when Pro skill re-roll is used for a specific block die.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandUseProReRollForBlock {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `proIndex`
    pub pro_index: i32,
}

impl ClientCommandUseProReRollForBlock {
    pub fn new(pro_index: i32) -> Self { Self { entropy: None, pro_index } }
    pub fn get_pro_index(&self) -> i32 { self.pro_index }

    /// Java: `ClientCommandUseProReRollForBlock.toJsonValue()` (calls `super.toJsonValue()` first).
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("proIndex".to_string(), serde_json::json!(self.pro_index));
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandUseProReRollForBlock.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            pro_index: json.get("proIndex").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
        }
    }
}

impl NetCommand for ClientCommandUseProReRollForBlock {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientUseProReRollForBlock
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn index_stored() {
        assert_eq!(ClientCommandUseProReRollForBlock::new(2).get_pro_index(), 2);
    }
    #[test]
    fn default_zero() {
        assert_eq!(ClientCommandUseProReRollForBlock::default().pro_index, 0);
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandUseProReRollForBlock::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandUseProReRollForBlock::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandUseProReRollForBlock::default());
        assert!(s.contains("ClientCommandUseProReRollForBlock"));
    }

    #[test]
    fn get_id_is_client_use_pro_re_roll_for_block() {
        assert_eq!(ClientCommandUseProReRollForBlock::new(0).get_id(), NetCommandId::ClientUseProReRollForBlock);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_pro_index() {
        let cmd = ClientCommandUseProReRollForBlock::new(2);
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientUseProReRollForBlock");
        assert_eq!(json["proIndex"], 2);
    }

    #[test]
    fn round_trip_with_index_and_entropy() {
        let mut cmd = ClientCommandUseProReRollForBlock::new(3);
        cmd.entropy = Some(6);
        let json = cmd.to_json_value();
        let restored = ClientCommandUseProReRollForBlock::from_json(&json);
        assert_eq!(restored.entropy, Some(6));
        assert_eq!(restored.pro_index, 3);
    }

    #[test]
    fn round_trip_with_default_index() {
        let cmd = ClientCommandUseProReRollForBlock::default();
        let json = cmd.to_json_value();
        let restored = ClientCommandUseProReRollForBlock::from_json(&json);
        assert_eq!(restored.pro_index, 0);
    }
}
