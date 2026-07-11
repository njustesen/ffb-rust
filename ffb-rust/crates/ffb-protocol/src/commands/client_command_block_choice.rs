use ffb_model::enums::NetCommandId;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandBlockChoice`.
/// Sent when the attacker selects which block die result to use.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandBlockChoice {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `fDiceIndex` — index of the chosen die result.
    pub dice_index: i32,
}

impl ClientCommandBlockChoice {
    pub fn new(dice_index: i32) -> Self {
        Self { entropy: None, dice_index }
    }

    /// Java: `getDiceIndex()`
    pub fn get_dice_index(&self) -> i32 { self.dice_index }

    /// Java: `ClientCommandBlockChoice.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("diceIndex".to_string(), serde_json::json!(self.dice_index));
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandBlockChoice.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            dice_index: json.get("diceIndex").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
        }
    }
}

impl NetCommand for ClientCommandBlockChoice {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientBlockChoice
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dice_index_stored() {
        let cmd = ClientCommandBlockChoice::new(2);
        assert_eq!(cmd.get_dice_index(), 2);
    }

    #[test]
    fn default_is_zero() {
        let cmd = ClientCommandBlockChoice::default();
        assert_eq!(cmd.dice_index, 0);
    }

    #[test]
    fn negative_index_stored() {
        let cmd = ClientCommandBlockChoice::new(-1);
        assert_eq!(cmd.get_dice_index(), -1);
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandBlockChoice::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandBlockChoice::default().clone();
    }

    #[test]
    fn get_id_is_client_block_choice() {
        assert_eq!(ClientCommandBlockChoice::new(0).get_id(), NetCommandId::ClientBlockChoice);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_dice_index() {
        let cmd = ClientCommandBlockChoice::new(2);
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientBlockChoice");
        assert_eq!(json["diceIndex"], 2);
    }

    #[test]
    fn round_trip_with_populated_data() {
        let mut cmd = ClientCommandBlockChoice::new(3);
        cmd.entropy = Some(11);
        let json = cmd.to_json_value();
        let restored = ClientCommandBlockChoice::from_json(&json);
        assert_eq!(restored.entropy, Some(11));
        assert_eq!(restored.get_dice_index(), 3);
    }

    #[test]
    fn round_trip_with_default_data() {
        let cmd = ClientCommandBlockChoice::default();
        let json = cmd.to_json_value();
        let restored = ClientCommandBlockChoice::from_json(&json);
        assert!(restored.entropy.is_none());
        assert_eq!(restored.get_dice_index(), 0);
    }
}
