/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::Stab.
use crate::model::skill::Skill;
use crate::enums::SkillCategory;

pub struct Stab {
    pub base: Skill,
}

impl Stab {
    pub fn new() -> Self {
        let base = Skill::new("Stab", SkillCategory::Extraordinary);
        Self { base }
    }
}

impl Default for Stab {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Stab {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(Stab::new().get_name(), "Stab");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(Stab::new().get_category(), SkillCategory::Extraordinary);
    }
}
