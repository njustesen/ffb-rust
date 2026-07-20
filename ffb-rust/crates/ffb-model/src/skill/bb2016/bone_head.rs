/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::BoneHead.
use crate::model::skill::skill::Skill;
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
// Mirrors ffb-java/ffb-server/src/test/java/com/fumbbl/ffb/server/skill tests.
// Java test targets bb2025 ("Bone Head", TRAIT); bb2016/BoneHead.java is "Bone-Head", EXTRAORDINARY.
mod tests {
    use super::*;
    use crate::enums::SkillId;

    #[test]
    fn name_is_bone_head() {
        assert_eq!(BoneHead::new().get_name(), "Bone-Head");
    }

    #[test]
    fn category_is_extraordinary() {
        assert_eq!(BoneHead::new().get_category(), SkillCategory::Extraordinary);
    }

    #[test]
    fn has_applies_confusion_property() {
        assert!(SkillId::BoneHead.properties().contains(&"appliesConfusion"));
    }
}
