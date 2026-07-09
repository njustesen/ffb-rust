/// 1:1 translation of com.fumbbl.ffb.skill.bb2025::PutTheBootIn.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct PutTheBootIn {
    pub base: Skill,
}

impl PutTheBootIn {
    pub fn new() -> Self {
        let base = Skill::new("Put the Boot In", SkillCategory::Devious);
        Self { base }
    }
}

impl Default for PutTheBootIn {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for PutTheBootIn {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(PutTheBootIn::new().get_name(), "Put the Boot In");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(PutTheBootIn::new().get_category(), SkillCategory::Devious);
    }
}
