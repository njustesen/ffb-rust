/// 1:1 translation of com.fumbbl.ffb.skill.mixed::PlagueRidden.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct PlagueRidden {
    pub base: Skill,
}

impl PlagueRidden {
    pub fn new() -> Self {
        let base = Skill::new("Plague Ridden", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for PlagueRidden {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for PlagueRidden {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mirrors ffb-java PlagueRiddenSkillTest.
    // Java `has_two_edition_annotations` is skipped (edition annotations are
    // covered elsewhere). Java `has_skill_properties_not_null` has no live Rust
    // equivalent (properties() returns a slice, which can never be null).

    #[test]
    fn name_is_plague_ridden() {
        assert_eq!(PlagueRidden::new().get_name(), "Plague Ridden");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(PlagueRidden::new().get_category(), SkillCategory::Trait);
    }
}
