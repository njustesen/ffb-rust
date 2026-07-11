use ffb_model::enums::NetCommandId;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of ClientCommandUseConsummateReRollForBlock (Java field: proIndex).
#[derive(Debug, Clone, Default)]
pub struct ClientCommandUseConsummateReRollForBlock {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    pub pro_index: i32,
}

impl ClientCommandUseConsummateReRollForBlock {
    pub fn new(pro_index: i32) -> Self {
        Self { entropy: None, pro_index }
    }

    pub fn get_pro_index(&self) -> i32 {
        self.pro_index
    }

    /// Java: `ClientCommandUseConsummateReRollForBlock.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("proIndex".to_string(), serde_json::json!(self.pro_index));
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandUseConsummateReRollForBlock.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            pro_index: json.get("proIndex").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
        }
    }
}

impl NetCommand for ClientCommandUseConsummateReRollForBlock {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientUseConsummateReRollForBlock
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_index() {
        let cmd = ClientCommandUseConsummateReRollForBlock::new(3);
        assert_eq!(cmd.get_pro_index(), 3);
    }

    #[test]
    fn default_is_zero() {
        let cmd = ClientCommandUseConsummateReRollForBlock::default();
        assert_eq!(cmd.get_pro_index(), 0);
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandUseConsummateReRollForBlock::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandUseConsummateReRollForBlock::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandUseConsummateReRollForBlock::default());
        assert!(s.contains("ClientCommandUseConsummateReRollForBlock"));
    }

    #[test]
    fn get_id_is_client_use_consummate_re_roll_for_block() {
        assert_eq!(
            ClientCommandUseConsummateReRollForBlock::default().get_id(),
            NetCommandId::ClientUseConsummateReRollForBlock
        );
    }

    #[test]
    fn to_json_value_has_net_command_id_and_pro_index() {
        let cmd = ClientCommandUseConsummateReRollForBlock::new(4);
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientUseConsummateReRollForBlock");
        assert_eq!(json["proIndex"], 4);
    }

    #[test]
    fn round_trip_with_index_and_entropy() {
        let mut cmd = ClientCommandUseConsummateReRollForBlock::new(7);
        cmd.entropy = Some(3);
        let json = cmd.to_json_value();
        let restored = ClientCommandUseConsummateReRollForBlock::from_json(&json);
        assert_eq!(restored.entropy, Some(3));
        assert_eq!(restored.get_pro_index(), 7);
    }

    #[test]
    fn round_trip_with_default_zero() {
        let cmd = ClientCommandUseConsummateReRollForBlock::default();
        let json = cmd.to_json_value();
        let restored = ClientCommandUseConsummateReRollForBlock::from_json(&json);
        assert_eq!(restored.get_pro_index(), 0);
    }
}
