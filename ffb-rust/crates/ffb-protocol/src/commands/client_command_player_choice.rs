use ffb_model::enums::NetCommandId;
use ffb_model::model::PlayerChoiceMode;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandPlayerChoice`.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandPlayerChoice {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `fPlayerChoiceMode`
    pub player_choice_mode: Option<PlayerChoiceMode>,
    /// Java: `fPlayerIds`
    pub player_ids: Vec<String>,
}

impl ClientCommandPlayerChoice {
    pub fn new() -> Self { Self::default() }

    pub fn with_mode(player_choice_mode: PlayerChoiceMode) -> Self {
        Self { entropy: None, player_choice_mode: Some(player_choice_mode), player_ids: Vec::new() }
    }

    pub fn get_player_choice_mode(&self) -> Option<PlayerChoiceMode> { self.player_choice_mode }
    pub fn get_player_ids(&self) -> &[String] { &self.player_ids }

    pub fn add_player_id(&mut self, id: impl Into<String>) {
        self.player_ids.push(id.into());
    }

    /// Java: `ClientCommandPlayerChoice.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        if let Some(mode) = self.player_choice_mode {
            map.insert("playerChoiceMode".to_string(), serde_json::json!(mode.get_name()));
        }
        map.insert("playerIds".to_string(), serde_json::json!(self.player_ids));
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandPlayerChoice.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            player_choice_mode: json.get("playerChoiceMode").and_then(|v| v.as_str()).and_then(PlayerChoiceMode::for_name),
            player_ids: json
                .get("playerIds")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default(),
        }
    }
}

impl NetCommand for ClientCommandPlayerChoice {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientPlayerChoice
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_stored_and_ids_added() {
        let mut cmd = ClientCommandPlayerChoice::with_mode(PlayerChoiceMode::BLOCK);
        cmd.add_player_id("p1");
        cmd.add_player_id("p2");
        assert_eq!(cmd.get_player_choice_mode(), Some(PlayerChoiceMode::BLOCK));
        assert_eq!(cmd.get_player_ids().len(), 2);
    }

    #[test]
    fn default_is_empty() {
        let cmd = ClientCommandPlayerChoice::new();
        assert!(cmd.player_choice_mode.is_none());
        assert!(cmd.player_ids.is_empty());
    }

    #[test]
    fn add_single_id_len_is_one() {
        let mut cmd = ClientCommandPlayerChoice::new();
        cmd.add_player_id("p99");
        assert_eq!(cmd.get_player_ids().len(), 1);
    }


    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandPlayerChoice::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandPlayerChoice::default().clone();
    }

    #[test]
    fn get_id_is_client_player_choice() {
        assert_eq!(ClientCommandPlayerChoice::new().get_id(), NetCommandId::ClientPlayerChoice);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_mode() {
        let mut cmd = ClientCommandPlayerChoice::with_mode(PlayerChoiceMode::CARD);
        cmd.add_player_id("p1");
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientPlayerChoice");
        assert_eq!(json["playerChoiceMode"], "card");
        assert_eq!(json["playerIds"], serde_json::json!(["p1"]));
    }

    #[test]
    fn round_trip_with_data() {
        let mut cmd = ClientCommandPlayerChoice::with_mode(PlayerChoiceMode::MVP);
        cmd.add_player_id("p1");
        cmd.add_player_id("p2");
        cmd.entropy = Some(8);
        let json = cmd.to_json_value();
        let restored = ClientCommandPlayerChoice::from_json(&json);
        assert_eq!(restored.entropy, Some(8));
        assert_eq!(restored.get_player_choice_mode(), Some(PlayerChoiceMode::MVP));
        assert_eq!(restored.get_player_ids(), &["p1".to_string(), "p2".to_string()]);
    }

    #[test]
    fn round_trip_default() {
        let cmd = ClientCommandPlayerChoice::default();
        let json = cmd.to_json_value();
        let restored = ClientCommandPlayerChoice::from_json(&json);
        assert!(restored.player_choice_mode.is_none());
        assert!(restored.player_ids.is_empty());
        assert!(restored.entropy.is_none());
    }
}
