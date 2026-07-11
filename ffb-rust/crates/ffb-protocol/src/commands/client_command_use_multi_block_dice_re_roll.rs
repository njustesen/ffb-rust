use ffb_model::enums::NetCommandId;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandUseMultiBlockDiceReRoll`.
/// Sent to re-roll specific dice in a multi-block action.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandUseMultiBlockDiceReRoll {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `diceIndexes`
    pub dice_indexes: Vec<i32>,
}

impl ClientCommandUseMultiBlockDiceReRoll {
    pub fn new() -> Self { Self::default() }
    pub fn with_indexes(dice_indexes: Vec<i32>) -> Self { Self { entropy: None, dice_indexes } }
    pub fn get_dice_indexes(&self) -> &[i32] { &self.dice_indexes }

    /// Java: `ClientCommandUseMultiBlockDiceReRoll.toJsonValue()` (calls `super.toJsonValue()` first).
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("blockDiceIndexes".to_string(), serde_json::json!(self.dice_indexes));
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandUseMultiBlockDiceReRoll.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        let dice_indexes = json
            .get("blockDiceIndexes")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_i64().map(|n| n as i32)).collect())
            .unwrap_or_default();
        Self { entropy: base.entropy, dice_indexes }
    }
}

impl NetCommand for ClientCommandUseMultiBlockDiceReRoll {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientUseMultiBlockDiceReRoll
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn indexes_stored() {
        let cmd = ClientCommandUseMultiBlockDiceReRoll::with_indexes(vec![0, 2]);
        assert_eq!(cmd.get_dice_indexes(), &[0, 2]);
    }
    #[test]
    fn default_empty() {
        assert!(ClientCommandUseMultiBlockDiceReRoll::new().dice_indexes.is_empty());
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandUseMultiBlockDiceReRoll::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandUseMultiBlockDiceReRoll::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandUseMultiBlockDiceReRoll::default());
        assert!(s.contains("ClientCommandUseMultiBlockDiceReRoll"));
    }

    #[test]
    fn get_id_is_client_use_multi_block_dice_re_roll() {
        assert_eq!(ClientCommandUseMultiBlockDiceReRoll::new().get_id(), NetCommandId::ClientUseMultiBlockDiceReRoll);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_indexes() {
        let cmd = ClientCommandUseMultiBlockDiceReRoll::with_indexes(vec![1, 3]);
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientUseMultiBlockDiceReRoll");
        assert_eq!(json["blockDiceIndexes"], serde_json::json!([1, 3]));
    }

    #[test]
    fn round_trip_with_indexes_and_entropy() {
        let mut cmd = ClientCommandUseMultiBlockDiceReRoll::with_indexes(vec![0, 1, 2]);
        cmd.entropy = Some(8);
        let json = cmd.to_json_value();
        let restored = ClientCommandUseMultiBlockDiceReRoll::from_json(&json);
        assert_eq!(restored.entropy, Some(8));
        assert_eq!(restored.dice_indexes, vec![0, 1, 2]);
    }

    #[test]
    fn round_trip_with_no_indexes() {
        let cmd = ClientCommandUseMultiBlockDiceReRoll::new();
        let json = cmd.to_json_value();
        let restored = ClientCommandUseMultiBlockDiceReRoll::from_json(&json);
        assert!(restored.dice_indexes.is_empty());
    }
}
