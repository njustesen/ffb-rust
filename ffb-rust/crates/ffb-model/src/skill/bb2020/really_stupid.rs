/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::ReallyStupid.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct ReallyStupid {
    pub base: Skill,
}

impl ReallyStupid {
    pub fn new() -> Self {
        let base = Skill::as_negative_trait("Really Stupid", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for ReallyStupid {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for ReallyStupid {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::SkillId;

    #[test]
    fn name_is_really_stupid() {
        assert_eq!(ReallyStupid::new().get_name(), "Really Stupid");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(ReallyStupid::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn has_applies_confusion_property() {
        assert!(SkillId::ReallyStupid.properties().contains(&"appliesConfusion"));
    }

    #[test]
    fn is_negative_trait() {
        assert!(ReallyStupid::new().is_negative_trait());
    }
}
