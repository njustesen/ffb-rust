/// 1:1 translation of com.fumbbl.ffb.model.skill.SkillDisplayInfo.
use crate::model::skill::skill::Skill;

/// Carries the display string, category classification, and the skill reference
/// used to render a player's skill list in the UI.
pub struct SkillDisplayInfo {
    info: String,
    category: DisplayCategory,
    skill: Skill,
}

/// Java inner enum `SkillDisplayInfo.Category`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DisplayCategory {
    Roster,
    Player,
    Temporary,
}

impl SkillDisplayInfo {
    /// `SkillDisplayInfo(String info, Category category, Skill skill)`.
    pub fn new(info: impl Into<String>, category: DisplayCategory, skill: Skill) -> Self {
        SkillDisplayInfo { info: info.into(), category, skill }
    }

    /// Java `getInfo()`.
    pub fn get_info(&self) -> &str {
        &self.info
    }

    /// Java `getCategory()`.
    pub fn get_category(&self) -> DisplayCategory {
        self.category
    }

    /// Java `getSkill()`.
    pub fn get_skill(&self) -> &Skill {
        &self.skill
    }
}
