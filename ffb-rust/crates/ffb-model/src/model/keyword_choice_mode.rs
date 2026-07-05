use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.KeywordChoiceMode.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeywordChoiceMode {
    GETTING_EVEN,
}

impl KeywordChoiceMode {
    pub fn get_name(self) -> &'static str {
        match self { KeywordChoiceMode::GETTING_EVEN => "gettingEven" }
    }

    pub fn get_dialog_header(self, player_name: &str) -> String {
        match self {
            KeywordChoiceMode::GETTING_EVEN =>
                format!("Select player type to get even with for {}", player_name),
        }
    }

    pub fn get_status_title(self) -> &'static str {
        match self { KeywordChoiceMode::GETTING_EVEN => "Getting Even" }
    }

    pub fn get_status_message(self) -> &'static str {
        match self { KeywordChoiceMode::GETTING_EVEN => "Waiting for coach to choose player type." }
    }

    pub fn for_name(name: &str) -> Option<Self> {
        match name { "gettingEven" => Some(KeywordChoiceMode::GETTING_EVEN), _ => None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn getting_even_get_name_is_camel_case() {
        assert_eq!(KeywordChoiceMode::GETTING_EVEN.get_name(), "gettingEven");
    }

    #[test]
    fn for_name_round_trips() {
        assert_eq!(KeywordChoiceMode::for_name("gettingEven"), Some(KeywordChoiceMode::GETTING_EVEN));
        assert_eq!(KeywordChoiceMode::for_name("invalid"), None);
    }
}
