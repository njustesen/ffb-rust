use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.SkillChoiceMode.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillChoiceMode {
    INTENSIVE_TRAINING,
    WISDOM_OF_THE_WHITE_DWARF,
}

impl SkillChoiceMode {
    pub fn get_name(self) -> &'static str {
        match self {
            SkillChoiceMode::INTENSIVE_TRAINING => "intensiveTraining",
            SkillChoiceMode::WISDOM_OF_THE_WHITE_DWARF => "wisdomOfTheWhiteDwarf",
        }
    }

    pub fn get_dialog_header(self, player_name: &str) -> String {
        match self {
            SkillChoiceMode::INTENSIVE_TRAINING =>
                format!("Select a primary skill for {}", player_name),
            SkillChoiceMode::WISDOM_OF_THE_WHITE_DWARF =>
                format!("Select a skill for{}", player_name),
        }
    }

    pub fn get_status_title(self) -> &'static str {
        match self {
            SkillChoiceMode::INTENSIVE_TRAINING => "Intensive Training",
            SkillChoiceMode::WISDOM_OF_THE_WHITE_DWARF => "Wisdom of the White Dwarf",
        }
    }

    pub fn get_status_message(self) -> &'static str {
        match self {
            SkillChoiceMode::INTENSIVE_TRAINING | SkillChoiceMode::WISDOM_OF_THE_WHITE_DWARF =>
                "Waiting for coach to choose Skill.",
        }
    }

    pub fn for_name(name: &str) -> Option<Self> {
        match name {
            "intensiveTraining" => Some(SkillChoiceMode::INTENSIVE_TRAINING),
            "wisdomOfTheWhiteDwarf" => Some(SkillChoiceMode::WISDOM_OF_THE_WHITE_DWARF),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_name_is_camel_case() {
        assert_eq!(SkillChoiceMode::INTENSIVE_TRAINING.get_name(), "intensiveTraining");
        assert_eq!(SkillChoiceMode::WISDOM_OF_THE_WHITE_DWARF.get_name(), "wisdomOfTheWhiteDwarf");
    }

    #[test]
    fn for_name_round_trips() {
        assert_eq!(SkillChoiceMode::for_name("intensiveTraining"), Some(SkillChoiceMode::INTENSIVE_TRAINING));
        assert_eq!(SkillChoiceMode::for_name("invalid"), None);
    }

    #[test]
    fn get_dialog_header_includes_player_name() {
        let header = SkillChoiceMode::INTENSIVE_TRAINING.get_dialog_header("Bob");
        assert!(header.contains("Bob"));
    }

    #[test]
    fn get_status_title_matches_both_variants() {
        assert_eq!(SkillChoiceMode::INTENSIVE_TRAINING.get_status_title(), "Intensive Training");
        assert_eq!(
            SkillChoiceMode::WISDOM_OF_THE_WHITE_DWARF.get_status_title(),
            "Wisdom of the White Dwarf"
        );
    }

    #[test]
    fn for_name_wisdom_round_trips_and_status_message_shared() {
        let mode = SkillChoiceMode::for_name("wisdomOfTheWhiteDwarf")
            .expect("wisdomOfTheWhiteDwarf must be recognized");
        assert_eq!(mode, SkillChoiceMode::WISDOM_OF_THE_WHITE_DWARF);
        // Both variants share the same status message
        assert_eq!(
            SkillChoiceMode::INTENSIVE_TRAINING.get_status_message(),
            SkillChoiceMode::WISDOM_OF_THE_WHITE_DWARF.get_status_message(),
        );
    }
}
