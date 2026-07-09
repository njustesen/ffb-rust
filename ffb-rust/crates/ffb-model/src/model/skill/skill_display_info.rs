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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::SkillCategory;

    fn make_skill(name: &str) -> Skill {
        Skill::new(name, SkillCategory::General)
    }

    #[test]
    fn new_stores_info_and_category() {
        let sdi = SkillDisplayInfo::new("Block", DisplayCategory::Roster, make_skill("Block"));
        assert_eq!(sdi.get_info(), "Block");
        assert_eq!(sdi.get_category(), DisplayCategory::Roster);
    }

    #[test]
    fn get_skill_returns_correct_name() {
        let sdi = SkillDisplayInfo::new("Dodge", DisplayCategory::Player, make_skill("Dodge"));
        assert_eq!(sdi.get_skill().get_name(), "Dodge");
    }

    #[test]
    fn temporary_category_stored() {
        let sdi = SkillDisplayInfo::new("Mighty Blow (+2)", DisplayCategory::Temporary, make_skill("Mighty Blow"));
        assert_eq!(sdi.get_category(), DisplayCategory::Temporary);
    }

    #[test]
    fn info_stored_as_given() {
        let sdi = SkillDisplayInfo::new("Tackle", DisplayCategory::Player, make_skill("Tackle"));
        assert_eq!(sdi.get_info(), "Tackle");
    }
}
