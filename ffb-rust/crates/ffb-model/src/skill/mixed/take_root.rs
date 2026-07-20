/// 1:1 translation of com.fumbbl.ffb.skill.mixed::TakeRoot.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct TakeRoot {
    pub base: Skill,
}

impl TakeRoot {
    pub fn new() -> Self {
        let base = Skill::as_negative_trait("Take Root", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for TakeRoot {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for TakeRoot {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mirrors ffb-java TakeRootSkillTest (written against the bb2016 class, where
    // the category is EXTRAORDINARY); category adapted to the mixed edition's
    // constructor (TRAIT). Property assertion verified against the mixed
    // edition's postConstruct, which registers becomesImmovable. The Java
    // `is_bb2016_edition` test is skipped (edition annotations are covered
    // elsewhere).

    #[test]
    fn name_is_take_root() {
        assert_eq!(TakeRoot::new().get_name(), "Take Root");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(TakeRoot::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn has_becomes_immovable_property() {
        assert!(crate::enums::SkillId::TakeRoot.properties().contains(&"becomesImmovable"));
    }

    // Additional Rust-side logic test beyond the Java test class.
    #[test]
    fn is_negative_trait() { assert!(TakeRoot::new().is_negative_trait()); }
}
