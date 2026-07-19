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
    fn name_is_correct() { assert_eq!(SavageBlow::new().get_name(), "Savage Blow"); }
    #[test]
    fn category_is_correct() { assert_eq!(SavageBlow::new().get_category(), SkillCategory::Trait); }
    #[test]
    fn usage_type_is_once_per_game() { assert_eq!(SavageBlow::new().skill_usage_type, SkillUsageType::OncePerGame); }
    #[test]
    fn registers_multi_block_dice_reroll_source() {
        let skill = SavageBlow::new();
        let source = skill.base.reroll_sources.get(&ReRolledAction::new("Multi Block Dice"));
        assert_eq!(source, Some(&ReRollSource::new("Savage Blow")));
    }
}
