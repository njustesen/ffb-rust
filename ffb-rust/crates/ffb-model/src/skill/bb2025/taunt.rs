/// 1:1 translation of com.fumbbl.ffb.skill.bb2025::Taunt.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Taunt {
    pub base: Skill,
}

impl Taunt {
    pub fn new() -> Self {
        let base = Skill::new("Taunt", SkillCategory::General);
        Self { base }
    }
}

impl Default for Taunt {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Taunt {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(Taunt::new().get_name(), "Taunt");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(Taunt::new().get_category(), SkillCategory::General);
    }
}
