/// 1:1 translation of com.fumbbl.ffb.skill.common::StandFirm.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct StandFirm {
    pub base: Skill,
}

impl StandFirm {
    pub fn new() -> Self {
        let base = Skill::new("Stand Firm", SkillCategory::Strength);
        Self { base }
    }
}

impl Default for StandFirm {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for StandFirm {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(StandFirm::new().get_name(), "Stand Firm"); }
    #[test]
    fn category_is_correct() { assert_eq!(StandFirm::new().get_category(), SkillCategory::Strength); }
}
