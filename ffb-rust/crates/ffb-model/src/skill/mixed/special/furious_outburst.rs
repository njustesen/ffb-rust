/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::FuriousOutburst.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct FuriousOutburst {
    pub base: Skill,
}

impl FuriousOutburst {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Furious Outburst", SkillCategory::Trait, SkillUsageType::OncePerHalf);
        Self { base }
    }
}

impl Default for FuriousOutburst {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for FuriousOutburst {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(FuriousOutburst::new().get_name(), "Furious Outburst"); }
    #[test]
    fn category_is_correct() { assert_eq!(FuriousOutburst::new().get_category(), SkillCategory::Trait); }
    #[test]
    fn usage_type_is_once_per_half() { assert_eq!(FuriousOutburst::new().get_skill_usage_type(), SkillUsageType::OncePerHalf); }
}
