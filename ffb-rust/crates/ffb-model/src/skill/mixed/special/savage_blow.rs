/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::SavageBlow.
use crate::model::skill::skill::Skill;
use crate::model::re_rolled_action::ReRolledAction;
use crate::enums::{SkillCategory, SkillUsageType, ReRollSource};

pub struct SavageBlow {
    pub base: Skill,
}

impl SavageBlow {
    pub fn new() -> Self {
        let mut base = Skill::with_usage_type("Savage Blow", SkillCategory::Trait, SkillUsageType::OncePerGame);
        // Java postConstruct: registerRerollSource(ReRolledActions.MULTI_BLOCK_DICE, ReRollSources.SAVAGE_BLOW);
        base.register_reroll_source(ReRolledAction::new("Multi Block Dice"), ReRollSource::new("Savage Blow"));
        Self { base }
    }
}

impl Default for SavageBlow {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for SavageBlow {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_savage_blow() {
        assert_eq!(SavageBlow::new().get_name(), "Savage Blow");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(SavageBlow::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn has_skill_properties_not_null() {
        // Java: assertNotNull(skill.getSkillProperties()); properties() always returns a valid slice.
        let _properties: &'static [&'static str] = crate::enums::SkillId::SavageBlow.properties();
    }

    #[test]
    fn usage_type_is_once_per_game() {
        assert_eq!(SavageBlow::new().skill_usage_type, SkillUsageType::OncePerGame);
    }
    #[test]
    fn has_savage_blow_reroll_source() {
        // Java: assertNotNull(skill.getRerollSource(ReRolledActions.MULTI_BLOCK_DICE)) —
        // mirrored against the live SkillId::reroll_sources() table (the per-struct
        // register_reroll_source map is inert).
        assert!(crate::enums::SkillId::SavageBlow.reroll_sources().iter().any(|(a, _)| *a == "MULTI_BLOCK_DICE"));
    }
}
