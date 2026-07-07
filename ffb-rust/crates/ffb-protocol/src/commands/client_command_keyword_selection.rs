/// 1:1 translation of com.fumbbl.ffb.net.commands.ClientCommandKeywordSelection.
use ffb_model::model::Keyword;

#[derive(Debug, Clone, Default)]
pub struct ClientCommandKeywordSelection {
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
}
