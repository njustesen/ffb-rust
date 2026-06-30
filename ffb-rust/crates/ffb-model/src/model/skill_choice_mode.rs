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
