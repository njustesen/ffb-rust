/// 1:1 translation of com.fumbbl.ffb.net.commands.ClientCommandKeywordSelection.
use ffb_model::enums::NetCommandId;
use ffb_model::model::Keyword;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

#[derive(Debug, Clone, Default)]
pub struct ClientCommandKeywordSelection {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `playerId`
    pub player_id: Option<String>,
    /// Java: `keywords`
    pub keywords: Vec<Keyword>,
}

impl ClientCommandKeywordSelection {
    pub fn new() -> Self {
        Self::default()
    }

    /// Java: `addKeyword(Keyword)`
    pub fn add_keyword(&mut self, keyword: Keyword) {
        self.keywords.push(keyword);
    }

    /// Java: `getPlayerId()`
    pub fn get_player_id(&self) -> Option<&str> {
        self.player_id.as_deref()
    }

    /// Java: `getKeywords()`
    pub fn get_keywords(&self) -> &[Keyword] {
        &self.keywords
    }

    /// Java: `ClientCommandKeywordSelection.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("playerId".to_string(), serde_json::json!(self.player_id));
        let names: Vec<&str> = self.keywords.iter().map(|k| k.get_name()).collect();
        map.insert("keywords".to_string(), serde_json::json!(names));
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandKeywordSelection.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        let keywords = json
            .get("keywords")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(Keyword::for_name).collect())
            .unwrap_or_default();
        Self {
            entropy: base.entropy,
            player_id: json.get("playerId").and_then(|v| v.as_str()).map(String::from),
            keywords,
        }
    }
}

impl NetCommand for ClientCommandKeywordSelection {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientKeywordSelection
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_no_player_id() {
        let cmd = ClientCommandKeywordSelection::new();
        assert!(cmd.get_player_id().is_none());
        assert!(cmd.get_keywords().is_empty());
    }

    #[test]
    fn add_keywords() {
        let mut cmd = ClientCommandKeywordSelection {
            entropy: None,
            player_id: Some("player_1".to_string()),
            keywords: vec![],
        };
        cmd.add_keyword(Keyword::GOBLIN);
        assert_eq!(cmd.get_player_id(), Some("player_1"));
        assert_eq!(cmd.get_keywords().len(), 1);
    }

    #[test]
    fn keywords_empty_by_default() {
        let cmd = ClientCommandKeywordSelection::default();
        assert!(cmd.get_keywords().is_empty());
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandKeywordSelection::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandKeywordSelection::default().clone();
    }

    #[test]
    fn get_id_is_client_keyword_selection() {
        assert_eq!(ClientCommandKeywordSelection::new().get_id(), NetCommandId::ClientKeywordSelection);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_keywords() {
        let mut cmd = ClientCommandKeywordSelection::new();
        cmd.player_id = Some("player_1".to_string());
        cmd.add_keyword(Keyword::GOBLIN);
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientKeywordSelection");
        assert_eq!(json["keywords"], serde_json::json!(["Goblin"]));
    }

    #[test]
    fn round_trip_with_keywords_and_entropy() {
        let mut cmd = ClientCommandKeywordSelection::new();
        cmd.entropy = Some(4);
        cmd.player_id = Some("player_1".to_string());
        cmd.add_keyword(Keyword::GOBLIN);
        cmd.add_keyword(Keyword::BIG_GUY);
        let json = cmd.to_json_value();
        let restored = ClientCommandKeywordSelection::from_json(&json);
        assert_eq!(restored.entropy, Some(4));
        assert_eq!(restored.get_player_id(), Some("player_1"));
        assert_eq!(restored.get_keywords(), &[Keyword::GOBLIN, Keyword::BIG_GUY]);
    }

    #[test]
    fn round_trip_with_empty_keywords() {
        let cmd = ClientCommandKeywordSelection::new();
        let json = cmd.to_json_value();
        let restored = ClientCommandKeywordSelection::from_json(&json);
        assert!(restored.get_keywords().is_empty());
        assert!(restored.get_player_id().is_none());
    }
}
