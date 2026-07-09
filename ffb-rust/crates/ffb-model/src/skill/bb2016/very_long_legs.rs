/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::VeryLongLegs.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct VeryLongLegs {
    pub base: Skill,
}

impl VeryLongLegs {
    pub fn new() -> Self {
        let base = Skill::new("Very Long Legs", SkillCategory::Mutation);
        Self { base }
    }
}

impl Default for VeryLongLegs {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for VeryLongLegs {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(VeryLongLegs::new().get_name(), "Very Long Legs");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(VeryLongLegs::new().get_category(), SkillCategory::Mutation);
    }
}
