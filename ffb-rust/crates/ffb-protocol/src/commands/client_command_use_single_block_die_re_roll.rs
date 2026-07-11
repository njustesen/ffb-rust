use ffb_model::enums::{NetCommandId, ReRollSource};
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandUseSingleBlockDieReRoll`.
/// Sent to re-roll a single block die result.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandUseSingleBlockDieReRoll {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `dieIndex`
    pub die_index: i32,
    /// Java: `reRollSource`
    pub re_roll_source: Option<ReRollSource>,
}

impl ClientCommandUseSingleBlockDieReRoll {
    pub fn new(die_index: i32) -> Self { Self { entropy: None, die_index, re_roll_source: None } }
    pub fn get_die_index(&self) -> i32 { self.die_index }
    pub fn get_re_roll_source(&self) -> Option<&ReRollSource> { self.re_roll_source.as_ref() }

    /// Java: `ClientCommandUseSingleBlockDieReRoll.toJsonValue()` (calls `super.toJsonValue()` first).
    /// `ReRollSource` is a plain named data class in Java (`JsonEnumWithNameOption` writes just
    /// its `getName()`), so only the `name` field round-trips over the wire.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("blockDieIndex".to_string(), serde_json::json!(self.die_index));
        if let Some(source) = &self.re_roll_source {
            map.insert("reRollSource".to_string(), serde_json::json!(source.name));
        }
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandUseSingleBlockDieReRoll.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            die_index: json.get("blockDieIndex").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            re_roll_source: json.get("reRollSource").and_then(|v| v.as_str()).map(ReRollSource::new),
        }
    }
}

impl NetCommand for ClientCommandUseSingleBlockDieReRoll {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientUseSingleBlockDieReRoll
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn die_index_stored() {
        assert_eq!(ClientCommandUseSingleBlockDieReRoll::new(1).get_die_index(), 1);
    }
    #[test]
    fn default_no_source() {
        assert!(ClientCommandUseSingleBlockDieReRoll::new(0).re_roll_source.is_none());
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandUseSingleBlockDieReRoll::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandUseSingleBlockDieReRoll::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandUseSingleBlockDieReRoll::default());
        assert!(s.contains("ClientCommandUseSingleBlockDieReRoll"));
    }

    #[test]
    fn get_id_is_client_use_single_block_die_re_roll() {
        assert_eq!(ClientCommandUseSingleBlockDieReRoll::new(0).get_id(), NetCommandId::ClientUseSingleBlockDieReRoll);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_die_index() {
        let cmd = ClientCommandUseSingleBlockDieReRoll::new(1);
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientUseSingleBlockDieReRoll");
        assert_eq!(json["blockDieIndex"], 1);
    }

    #[test]
    fn round_trip_with_source_and_entropy() {
        let mut cmd = ClientCommandUseSingleBlockDieReRoll::new(2);
        cmd.entropy = Some(9);
        cmd.re_roll_source = Some(ReRollSource::new("TRR"));
        let json = cmd.to_json_value();
        let restored = ClientCommandUseSingleBlockDieReRoll::from_json(&json);
        assert_eq!(restored.entropy, Some(9));
        assert_eq!(restored.die_index, 2);
        assert_eq!(restored.re_roll_source, Some(ReRollSource::new("TRR")));
    }

    #[test]
    fn round_trip_with_no_source() {
        let cmd = ClientCommandUseSingleBlockDieReRoll::new(0);
        let json = cmd.to_json_value();
        let restored = ClientCommandUseSingleBlockDieReRoll::from_json(&json);
        assert!(restored.re_roll_source.is_none());
    }
}
