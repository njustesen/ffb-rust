/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::BoneHead.
use crate::model::skill::Skill;
use crate::enums::SkillCategory;

pub struct BoneHead {
    pub base: Skill,
}

impl BoneHead {
    pub fn new() -> Self {
        let base = Skill::new("Bone-Head", SkillCategory::Extraordinary);
        Self { base }
    }
}

impl Default for BoneHead {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for BoneHead {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(BoneHead::new().get_name(), "Bone-Head");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(BoneHead::new().get_category(), SkillCategory::Extraordinary);
    }
}
