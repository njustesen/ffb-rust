/// 1:1 translation of com.fumbbl.ffb.skill.bb2025::Insignificant.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Insignificant {
    pub base: Skill,
}

impl Insignificant {
    pub fn new() -> Self {
        let base = Skill::as_negative_trait("Insignificant", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for Insignificant {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Insignificant {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_insignificant() {
        assert_eq!(Insignificant::new().get_name(), "Insignificant");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(Insignificant::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn skill_properties_are_not_null() {
        // Java: assertNotNull(skill.getSkillProperties()); the bb2025 Java postConstruct
        // registers no NamedProperties, so the live SkillId table must be empty here.
        assert!(crate::enums::SkillId::Insignificant.properties().is_empty());
    }

    #[test]
    fn is_negative_trait() {
        assert!(Insignificant::new().is_negative_trait());
    }
}
