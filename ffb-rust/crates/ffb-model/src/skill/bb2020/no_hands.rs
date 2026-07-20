/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::NoHands.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct NoHands {
    pub base: Skill,
}

impl NoHands {
    pub fn new() -> Self {
        let base = Skill::new("No Hands", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for NoHands {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for NoHands {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::SkillId;

    // bb2020/NoHands is SkillCategory.TRAIT (the bb2016 test's EXTRAORDINARY is bb2016-only).

    #[test]
    fn name_is_no_hands() {
        assert_eq!(NoHands::new().get_name(), "No Hands");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(NoHands::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn has_prevent_catch_property() {
        assert!(SkillId::NoHands.properties().contains(&"preventCatch"));
    }
}
