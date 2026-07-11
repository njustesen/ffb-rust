use ffb_model::enums::NetCommandId;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandArgueTheCall`.
/// Sent when a coach argues the call for one or more ejected players.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandArgueTheCall {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `fPlayerIds`
    pub player_ids: Vec<String>,
}

impl ClientCommandArgueTheCall {
    pub fn new() -> Self { Self::default() }

    pub fn with_player_id(player_id: impl Into<String>) -> Self {
        Self { entropy: None, player_ids: vec![player_id.into()] }
    }

    pub fn with_player_ids(player_ids: Vec<String>) -> Self {
        Self { entropy: None, player_ids }
    }

    pub fn get_player_ids(&self) -> &[String] { &self.player_ids }

    /// Java: `ClientCommandArgueTheCall.toJsonValue()` (calls `super.toJsonValue()` first).
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("playerIds".to_string(), serde_json::json!(self.player_ids));
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandArgueTheCall.initFrom(source, jsonValue)` — `addPlayerIds` filters
    /// out blank/empty ids (`StringTool.isProvided`), mirrored here.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        let player_ids = json
            .get("playerIds")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default();
        Self { entropy: base.entropy, player_ids }
    }
}

impl NetCommand for ClientCommandArgueTheCall {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientArgueTheCall
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_empty_player_ids() {
        let cmd = ClientCommandArgueTheCall::new();
        assert!(cmd.get_player_ids().is_empty());
    }

    #[test]
    fn with_player_id_stores_id() {
        let cmd = ClientCommandArgueTheCall::with_player_id("p1");
        assert_eq!(cmd.get_player_ids(), &["p1"]);
    }

    #[test]
    fn with_player_ids_stores_all() {
        let cmd = ClientCommandArgueTheCall::with_player_ids(vec!["a".into(), "b".into()]);
        assert_eq!(cmd.get_player_ids().len(), 2);
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandArgueTheCall::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandArgueTheCall::default().clone();
    }

    #[test]
    fn get_id_is_client_argue_the_call() {
        assert_eq!(ClientCommandArgueTheCall::new().get_id(), NetCommandId::ClientArgueTheCall);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_player_ids() {
        let cmd = ClientCommandArgueTheCall::with_player_ids(vec!["a".into(), "b".into()]);
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientArgueTheCall");
        assert_eq!(json["playerIds"], serde_json::json!(["a", "b"]));
    }

    #[test]
    fn round_trip_with_populated_fields() {
        let mut cmd = ClientCommandArgueTheCall::with_player_ids(vec!["a".into(), "b".into()]);
        cmd.entropy = Some(2);
        let json = cmd.to_json_value();
        let restored = ClientCommandArgueTheCall::from_json(&json);
        assert_eq!(restored.entropy, Some(2));
        assert_eq!(restored.get_player_ids(), &["a", "b"]);
    }

    #[test]
    fn round_trip_with_default_data() {
        let cmd = ClientCommandArgueTheCall::new();
        let json = cmd.to_json_value();
        let restored = ClientCommandArgueTheCall::from_json(&json);
        assert!(restored.get_player_ids().is_empty());
    }
}
