/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::PrehensileTail.
use crate::model::skill::Skill;
use crate::enums::SkillCategory;

pub struct PrehensileTail {
    pub base: Skill,
}

impl PrehensileTail {
    pub fn new() -> Self {
        let base = Skill::new("Prehensile Tail", SkillCategory::Mutation);
        Self { base }
    }
}

impl Default for PrehensileTail {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for PrehensileTail {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(PrehensileTail::new().get_name(), "Prehensile Tail");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(PrehensileTail::new().get_category(), SkillCategory::Mutation);
    }
}
