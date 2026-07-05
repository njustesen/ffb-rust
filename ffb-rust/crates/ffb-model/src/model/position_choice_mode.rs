use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.PositionChoiceMode.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PositionChoiceMode {
    RAISE_DEAD,
}

impl PositionChoiceMode {
    pub fn get_name(self) -> &'static str {
        match self { PositionChoiceMode::RAISE_DEAD => "raiseDead" }
    }

    pub fn get_dialog_header(self) -> &'static str {
        match self { PositionChoiceMode::RAISE_DEAD => "Select position for raised player" }
    }

    pub fn get_status_title(self) -> &'static str {
        match self { PositionChoiceMode::RAISE_DEAD => "Raise Dead" }
    }

    pub fn get_status_message(self) -> &'static str {
        match self { PositionChoiceMode::RAISE_DEAD => "Waiting for coach to choose position." }
    }

    pub fn for_name(name: &str) -> Option<Self> {
        match name { "raiseDead" => Some(PositionChoiceMode::RAISE_DEAD), _ => None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raise_dead_get_name_is_camel_case() {
        assert_eq!(PositionChoiceMode::RAISE_DEAD.get_name(), "raiseDead");
    }

    #[test]
    fn for_name_round_trips() {
        assert_eq!(PositionChoiceMode::for_name("raiseDead"), Some(PositionChoiceMode::RAISE_DEAD));
        assert_eq!(PositionChoiceMode::for_name("invalid"), None);
    }
}
