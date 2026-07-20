/// 1:1 translation of com.fumbbl.ffb.skill.bb2025::MonstrousMouth.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct MonstrousMouth {
    pub base: Skill,
}

impl MonstrousMouth {
    pub fn new() -> Self {
        let base = Skill::new("Monstrous Mouth", SkillCategory::Mutation);
        Self { base }
    }
}

impl Default for MonstrousMouth {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for MonstrousMouth {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_monstrous_mouth() {
        assert_eq!(MonstrousMouth::new().get_name(), "Monstrous Mouth");
    }

    #[test]
    fn category_is_mutation() {
        assert_eq!(MonstrousMouth::new().get_category(), SkillCategory::Mutation);
    }

    #[test]
    fn skill_properties_are_not_null() {
        // Java: assertNotNull(skill.getSkillProperties()); the bb2025 Java postConstruct
        // registers NamedProperties, so the live SkillId table must be populated.
        assert!(!crate::enums::SkillId::MonstrousMouth.properties().is_empty());
    }
}
