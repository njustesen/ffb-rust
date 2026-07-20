/// 1:1 translation of com.fumbbl.ffb.skill.common::Catch.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Catch {
    pub base: Skill,
}

impl Catch {
    pub fn new() -> Self {
        let base = Skill::new("Catch", SkillCategory::Agility);
        Self { base }
    }
}

impl Default for Catch {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Catch {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_catch() {
        assert_eq!(Catch::new().get_name(), "Catch");
    }

    #[test]
    fn category_is_agility() {
        assert_eq!(Catch::new().get_category(), SkillCategory::Agility);
    }

    #[test]
    fn has_no_named_properties() {
        // Catch uses a ReRollSource rather than NamedProperties
        assert!(crate::enums::SkillId::Catch.properties().is_empty());
    }

    #[test]
    fn has_catch_reroll_source() {
        // Java: assertNotNull(skill.getRerollSource(ReRolledActions.CATCH)) —
        // mirrored against the live SkillId::reroll_sources() table.
        assert!(crate::enums::SkillId::Catch.reroll_sources().iter().any(|(a, _)| *a == "CATCH"));
    }
}
