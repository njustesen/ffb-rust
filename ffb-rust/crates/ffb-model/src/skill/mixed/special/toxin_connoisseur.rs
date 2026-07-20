/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::ToxinConnoisseur.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct ToxinConnoisseur {
    pub base: Skill,
}

impl ToxinConnoisseur {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Toxin Connoisseur", SkillCategory::Trait, SkillUsageType::OncePerGame);
        Self { base }
    }
}

impl Default for ToxinConnoisseur {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for ToxinConnoisseur {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_toxin_connoisseur() {
        assert_eq!(ToxinConnoisseur::new().get_name(), "Toxin Connoisseur");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(ToxinConnoisseur::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn skill_properties_are_not_null() {
        // Java: assertNotNull(skill.getSkillProperties()); properties() always returns a valid slice.
        let _properties: &'static [&'static str] = crate::enums::SkillId::ToxinConnoisseur.properties();
    }

    #[test]
    fn class_name_is_toxin_connoisseur() {
        // Java: assertEquals("ToxinConnoisseur", skill.getClass().getSimpleName());
        assert!(std::any::type_name::<ToxinConnoisseur>().ends_with("::ToxinConnoisseur"));
    }

    #[test]
    fn usage_type_is_once_per_game() {
        assert_eq!(ToxinConnoisseur::new().skill_usage_type, SkillUsageType::OncePerGame);
    }
}
