/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::ViciousVines.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct ViciousVines {
    pub base: Skill,
}

impl ViciousVines {
    pub fn new() -> Self {
        let base = Skill::new("Vicious Vines", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for ViciousVines {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for ViciousVines {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(ViciousVines::new().get_name(), "Vicious Vines"); }
    #[test]
    fn category_is_correct() { assert_eq!(ViciousVines::new().get_category(), SkillCategory::Trait); }
}
