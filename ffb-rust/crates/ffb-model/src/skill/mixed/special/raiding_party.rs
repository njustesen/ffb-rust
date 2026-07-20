/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::RaidingParty.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct RaidingParty {
    pub base: Skill,
}

impl RaidingParty {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Raiding Party", SkillCategory::Trait, SkillUsageType::OncePerDrive);
        Self { base }
    }
}

impl Default for RaidingParty {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for RaidingParty {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_raiding_party() {
        assert_eq!(RaidingParty::new().get_name(), "Raiding Party");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(RaidingParty::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn has_skill_properties_not_null() {
        // Java: assertNotNull(skill.getSkillProperties()); properties() always returns a valid slice.
        let _properties: &'static [&'static str] = crate::enums::SkillId::RaidingParty.properties();
    }

    #[test]
    fn usage_type_is_once_per_drive() {
        assert_eq!(RaidingParty::new().skill_usage_type, SkillUsageType::OncePerDrive);
    }
}
