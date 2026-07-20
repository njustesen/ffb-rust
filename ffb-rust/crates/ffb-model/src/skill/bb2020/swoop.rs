/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::Swoop.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Swoop {
    pub base: Skill,
}

impl Swoop {
    pub fn new() -> Self {
        let base = Skill::new("Swoop", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for Swoop {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Swoop {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::SkillId;

    // bb2020/Swoop is SkillCategory.TRAIT (the bb2016 test's EXTRAORDINARY is bb2016-only).

    #[test]
    fn name_is_swoop() {
        assert_eq!(Swoop::new().get_name(), "Swoop");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(Swoop::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn has_prevent_stunty_dodge_modifier_property() {
        assert!(SkillId::Swoop.properties().contains(&"preventStuntyDodgeModifier"));
    }
}
