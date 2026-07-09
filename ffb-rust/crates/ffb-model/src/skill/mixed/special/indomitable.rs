/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::Indomitable.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Indomitable {
    pub base: Skill,
}

impl Indomitable {
    pub fn new() -> Self {
        let base = Skill::new("Indomitable", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for Indomitable {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Indomitable {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(Indomitable::new().get_name(), "Indomitable"); }
    #[test]
    fn category_is_correct() { assert_eq!(Indomitable::new().get_category(), SkillCategory::Trait); }
}
