/// 1:1 translation of com.fumbbl.ffb.net.commands.ClientCommandJourneymen.
use ffb_model::enums::NetCommandId;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

#[derive(Debug, Clone, Default)]
pub struct ClientCommandJourneymen {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `fSlots`
    pub slots: Vec<i32>,
    /// Java: `fPositionIds`
    pub position_ids: Vec<String>,
}

impl ClientCommandJourneymen {
    pub fn new() -> Self {
        Self::default()
    }

    /// Java: `addSlot(int)`
    pub fn add_slot(&mut self, slot: i32) {
        self.slots.push(slot);
    }

    /// Java: `addPositionId(String)`
    pub fn add_position_id(&mut self, position_id: String) {
        self.position_ids.push(position_id);
    }

    /// Java: `getSlots()`
    pub fn get_slots(&self) -> &[i32] {
        &self.slots
    }

    /// Java: `getPositionIds()`
    pub fn get_position_ids(&self) -> &[String] {
        &self.position_ids
    }

    /// Java: `ClientCommandJourneymen.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("positionIds".to_string(), serde_json::json!(self.position_ids));
        map.insert("slots".to_string(), serde_json::json!(self.slots));
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandJourneymen.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        let position_ids = json
            .get("positionIds")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();
        let slots = json
            .get("slots")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_i64().map(|i| i as i32)).collect())
            .unwrap_or_default();
        Self {
            entropy: base.entropy,
            slots,
            position_ids,
        }
    }
}

impl NetCommand for ClientCommandJourneymen {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientJourneymen
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_slots_and_positions() {
        let mut cmd = ClientCommandJourneymen::new();
        cmd.add_slot(1);
        cmd.add_slot(2);
        cmd.add_position_id("pos_lineman".to_string());
        assert_eq!(cmd.get_slots(), &[1, 2]);
        assert_eq!(cmd.get_position_ids(), &["pos_lineman"]);
    }

    #[test]
    fn default_empty_vecs() {
        let cmd = ClientCommandJourneymen::new();
        assert!(cmd.get_slots().is_empty());
        assert!(cmd.get_position_ids().is_empty());
    }

    #[test]
    fn slots_can_hold_multiple() {
        let mut cmd = ClientCommandJourneymen::new();
        cmd.add_slot(5);
        cmd.add_slot(6);
        assert_eq!(cmd.get_slots().len(), 2);
    }


    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandJourneymen::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandJourneymen::default().clone();
    }

    #[test]
    fn get_id_is_client_journeymen() {
        assert_eq!(ClientCommandJourneymen::new().get_id(), NetCommandId::ClientJourneymen);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_slots() {
        let mut cmd = ClientCommandJourneymen::new();
        cmd.add_slot(3);
        cmd.add_position_id("pos_1".to_string());
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientJourneymen");
        assert_eq!(json["slots"], serde_json::json!([3]));
        assert_eq!(json["positionIds"], serde_json::json!(["pos_1"]));
    }

    #[test]
    fn round_trip_with_slots_and_entropy() {
        let mut cmd = ClientCommandJourneymen::new();
        cmd.entropy = Some(2);
        cmd.add_slot(1);
        cmd.add_slot(2);
        cmd.add_position_id("pos_lineman".to_string());
        let json = cmd.to_json_value();
        let restored = ClientCommandJourneymen::from_json(&json);
        assert_eq!(restored.entropy, Some(2));
        assert_eq!(restored.get_slots(), &[1, 2]);
        assert_eq!(restored.get_position_ids(), &["pos_lineman"]);
    }

    #[test]
    fn round_trip_with_empty_vecs() {
        let cmd = ClientCommandJourneymen::new();
        let json = cmd.to_json_value();
        let restored = ClientCommandJourneymen::from_json(&json);
        assert!(restored.get_slots().is_empty());
        assert!(restored.get_position_ids().is_empty());
    }
}
