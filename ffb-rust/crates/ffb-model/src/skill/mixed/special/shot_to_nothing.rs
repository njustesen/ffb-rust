/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::ShotToNothing.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct ShotToNothing {
    pub base: Skill,
}

impl ShotToNothing {
    pub fn new() -> Self {
        let mut base = Skill::with_usage_type("Shot to Nothing", SkillCategory::Trait, SkillUsageType::OncePerGame);
        // Java postConstruct: setEnhancements(new TemporaryEnhancements().withSkills(Collections.singleton(new SkillClassWithValue(HailMaryPass.class))));
        base.set_enhancements("HailMaryPass".to_string());
        Self { base }
    }
}

impl Default for ShotToNothing {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for ShotToNothing {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_shot_to_nothing() {
        assert_eq!(ShotToNothing::new().get_name(), "Shot to Nothing");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(ShotToNothing::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn has_skill_properties_not_null() {
        // Java: assertNotNull(skill.getSkillProperties()); properties() always returns a valid slice.
        let _properties: &'static [&'static str] = crate::enums::SkillId::ShotToNothing.properties();
    }

    #[test]
    fn usage_type_is_once_per_game() {
        assert_eq!(ShotToNothing::new().skill_usage_type, SkillUsageType::OncePerGame);
    }
    #[test]
    fn grants_hail_mary_pass_enhancement() {
        assert_eq!(ShotToNothing::new().base.get_enhancements(), Some(&"HailMaryPass".to_string()));
    }
}
