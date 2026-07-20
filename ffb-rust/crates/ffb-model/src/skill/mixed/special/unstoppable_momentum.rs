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
    fn name_is_unstoppable_momentum() {
        assert_eq!(UnstoppableMomentum::new().get_name(), "Unstoppable Momentum");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(UnstoppableMomentum::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn has_skill_properties_not_null() {
        // Java: assertNotNull(skill.getSkillProperties()); properties() always returns a valid slice.
        let _properties: &'static [&'static str] = crate::enums::SkillId::UnstoppableMomentum.properties();
    }
    #[test]
    fn has_unstoppable_momentum_reroll_source() {
        // Java: assertNotNull(skill.getRerollSource(ReRolledActions.SINGLE_BLOCK_DIE)) —
        // mirrored against the live SkillId::reroll_sources() table (the per-struct
        // register_reroll_source map is inert).
        assert!(crate::enums::SkillId::UnstoppableMomentum.reroll_sources().iter().any(|(a, _)| *a == "SINGLE_BLOCK_DIE"));
    }
}
