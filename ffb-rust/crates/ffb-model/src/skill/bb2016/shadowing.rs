/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::Shadowing.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Shadowing {
    pub base: Skill,
}

impl Shadowing {
    pub fn new() -> Self {
        let base = Skill::new("Shadowing", SkillCategory::General);
        Self { base }
    }
}

impl Default for Shadowing {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Shadowing {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(Shadowing::new().get_name(), "Shadowing");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(Shadowing::new().get_category(), SkillCategory::General);
    }
}
