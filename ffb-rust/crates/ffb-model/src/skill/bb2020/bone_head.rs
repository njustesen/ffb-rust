/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::BoneHead.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct BoneHead {
    pub base: Skill,
}

impl BoneHead {
    pub fn new() -> Self {
        let base = Skill::as_negative_trait("Bone Head", SkillCategory::Trait);
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
    use crate::enums::SkillId;

    #[test]
    fn name_is_bone_head() {
        assert_eq!(BoneHead::new().get_name(), "Bone Head");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(BoneHead::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn has_applies_confusion_property() {
        assert!(SkillId::BoneHead.properties().contains(&"appliesConfusion"));
    }

    #[test]
    fn is_negative_trait() {
        assert!(BoneHead::new().is_negative_trait());
    }
}
