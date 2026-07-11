/// 1:1 translation of com.fumbbl.ffb.net.commands.ClientCommandBlockOrReRollChoiceForTarget.
use ffb_model::enums::{NetCommandId, ReRollSource};
use ffb_model::factory::re_roll_source_factory::ReRollSourceFactory;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

#[derive(Debug, Clone)]
pub struct ClientCommandBlockOrReRollChoiceForTarget {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `targetId`
    pub target_id: Option<String>,
    /// Java: `selectedIndex` — defaults to -1 in Java
    pub selected_index: i32,
    /// Java: `proIndex`
    pub pro_index: i32,
    /// Java: `reRollSource`
    pub re_roll_source: Option<ReRollSource>,
    /// Java: `anyDiceIndexes`
    pub any_dice_indexes: Vec<i32>,
}

impl Default for ClientCommandBlockOrReRollChoiceForTarget {
    fn default() -> Self {
        Self {
            entropy: None,
            target_id: None,
            selected_index: -1,
            pro_index: 0,
            re_roll_source: None,
            any_dice_indexes: vec![],
        }
    }
}

impl ClientCommandBlockOrReRollChoiceForTarget {
    pub fn new() -> Self {
        Self::default()
    }

    /// Java: `getTargetId()`
    pub fn get_target_id(&self) -> Option<&str> {
        self.target_id.as_deref()
    }

    /// Java: `getSelectedIndex()`
    pub fn get_selected_index(&self) -> i32 {
        self.selected_index
    }

    /// Java: `getProIndex()`
    pub fn get_pro_index(&self) -> i32 {
        self.pro_index
    }

    /// Java: `getReRollSource()`
    pub fn get_re_roll_source(&self) -> Option<&ReRollSource> {
        self.re_roll_source.as_ref()
    }

    /// Java: `getAnyDiceIndexes()`
    pub fn get_any_dice_indexes(&self) -> &[i32] {
        &self.any_dice_indexes
    }

    /// Java: `ClientCommandBlockOrReRollChoiceForTarget.toJsonValue()`.
    /// Note: `IJsonOption.RE_ROLL_SOURCE` (`JsonEnumWithNameOption`) serializes an
    /// `INamedObject` as just its name string (via `UtilJson.toJsonValue`), so we
    /// write/read `re_roll_source.name` rather than a full nested object.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        if let Some(re_roll_source) = &self.re_roll_source {
            map.insert("reRollSource".to_string(), serde_json::json!(re_roll_source.name));
        }
        if let Some(target_id) = &self.target_id {
            map.insert("playerId".to_string(), serde_json::json!(target_id));
        }
        map.insert("diceIndex".to_string(), serde_json::json!(self.selected_index));
        map.insert("proIndex".to_string(), serde_json::json!(self.pro_index));
        if !self.any_dice_indexes.is_empty() {
            map.insert("reRolledDice".to_string(), serde_json::json!(self.any_dice_indexes));
        }
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandBlockOrReRollChoiceForTarget.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        let re_roll_source = json
            .get("reRollSource")
            .and_then(|v| v.as_str())
            .and_then(|name| ReRollSourceFactory::default().for_name(name));
        let any_dice_indexes = json
            .get("reRolledDice")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_i64()).map(|n| n as i32).collect())
            .unwrap_or_default();
        Self {
            entropy: base.entropy,
            target_id: json.get("playerId").and_then(|v| v.as_str()).map(String::from),
            selected_index: json.get("diceIndex").and_then(|v| v.as_i64()).unwrap_or(-1) as i32,
            pro_index: json.get("proIndex").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            re_roll_source,
            any_dice_indexes,
        }
    }
}

impl NetCommand for ClientCommandBlockOrReRollChoiceForTarget {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientBlockOrReRollChoiceForTarget
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_selected_index_is_minus_one() {
        let cmd = ClientCommandBlockOrReRollChoiceForTarget::new();
        assert_eq!(cmd.get_selected_index(), -1);
    }

    #[test]
    fn stores_target_id_and_any_dice_indexes() {
        let cmd = ClientCommandBlockOrReRollChoiceForTarget {
            entropy: None,
            target_id: Some("target_1".to_string()),
            selected_index: 2,
            pro_index: 1,
            re_roll_source: None,
            any_dice_indexes: vec![0, 2],
        };
        assert_eq!(cmd.get_target_id(), Some("target_1"));
        assert_eq!(cmd.get_selected_index(), 2);
        assert_eq!(cmd.get_any_dice_indexes(), &[0, 2]);
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandBlockOrReRollChoiceForTarget::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_roundtrip() {
        let cmd = ClientCommandBlockOrReRollChoiceForTarget::default();
        let _ = cmd.clone();
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandBlockOrReRollChoiceForTarget::default().clone();
    }

    #[test]
    fn get_id_is_client_block_or_re_roll_choice_for_target() {
        assert_eq!(
            ClientCommandBlockOrReRollChoiceForTarget::new().get_id(),
            NetCommandId::ClientBlockOrReRollChoiceForTarget
        );
    }

    #[test]
    fn to_json_value_has_net_command_id_and_player_id() {
        let cmd = ClientCommandBlockOrReRollChoiceForTarget {
            target_id: Some("target_1".to_string()),
            ..Default::default()
        };
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientBlockOrReRollChoiceForTarget");
        assert_eq!(json["playerId"], "target_1");
    }

    #[test]
    fn round_trip_with_populated_data() {
        let mut cmd = ClientCommandBlockOrReRollChoiceForTarget {
            target_id: Some("target_1".to_string()),
            selected_index: 2,
            pro_index: 1,
            re_roll_source: ReRollSourceFactory::default().for_name("Pro"),
            any_dice_indexes: vec![0, 2],
            ..Default::default()
        };
        cmd.entropy = Some(4);
        let json = cmd.to_json_value();
        let restored = ClientCommandBlockOrReRollChoiceForTarget::from_json(&json);
        assert_eq!(restored.entropy, Some(4));
        assert_eq!(restored.get_target_id(), Some("target_1"));
        assert_eq!(restored.get_selected_index(), 2);
        assert_eq!(restored.get_pro_index(), 1);
        assert!(restored.get_re_roll_source().is_some());
        assert_eq!(restored.get_any_dice_indexes(), &[0, 2]);
    }

    #[test]
    fn round_trip_with_default_data() {
        let cmd = ClientCommandBlockOrReRollChoiceForTarget::default();
        let json = cmd.to_json_value();
        let restored = ClientCommandBlockOrReRollChoiceForTarget::from_json(&json);
        assert!(restored.entropy.is_none());
        assert!(restored.target_id.is_none());
        assert_eq!(restored.selected_index, -1);
        assert_eq!(restored.pro_index, 0);
        assert!(restored.re_roll_source.is_none());
        assert!(restored.any_dice_indexes.is_empty());
    }
}
