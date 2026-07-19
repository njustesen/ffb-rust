/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::UnstoppableMomentum.
use crate::model::skill::skill::Skill;
use crate::model::re_rolled_action::ReRolledAction;
use crate::enums::{SkillCategory, ReRollSource};

pub struct UnstoppableMomentum {
    pub base: Skill,
}

impl UnstoppableMomentum {
    pub fn new() -> Self {
        let mut base = Skill::new("Unstoppable Momentum", SkillCategory::Trait);
        // Java postConstruct: registerRerollSource(ReRolledActions.SINGLE_BLOCK_DIE, ReRollSources.UNSTOPPABLE_MOMENTUM);
        base.register_reroll_source(ReRolledAction::new("Single Block Die"), ReRollSource::new("Unstoppable Momentum"));
        Self { base }
    }
}

impl Default for UnstoppableMomentum {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for UnstoppableMomentum {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(UnstoppableMomentum::new().get_name(), "Unstoppable Momentum"); }
    #[test]
    fn category_is_correct() { assert_eq!(UnstoppableMomentum::new().get_category(), SkillCategory::Trait); }
    #[test]
    fn registers_single_block_die_reroll_source() {
        let skill = UnstoppableMomentum::new();
        let source = skill.base.reroll_sources.get(&ReRolledAction::new("Single Block Die"));
        assert_eq!(source, Some(&ReRollSource::new("Unstoppable Momentum")));
    }
}
