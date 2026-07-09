/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::PilingOn.
use crate::model::skill::Skill;
use crate::enums::SkillCategory;

pub struct PilingOn {
    pub base: Skill,
}

impl PilingOn {
    pub fn new() -> Self {
        let base = Skill::new("Piling On", SkillCategory::Strength);
        Self { base }
    }
}

impl Default for PilingOn {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for PilingOn {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(PilingOn::new().get_name(), "Piling On");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(PilingOn::new().get_category(), SkillCategory::Strength);
    }
}
