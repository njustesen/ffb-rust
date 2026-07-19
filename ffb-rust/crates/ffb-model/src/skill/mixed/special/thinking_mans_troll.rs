/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::ThinkingMansTroll.
use crate::model::skill::skill::Skill;
use crate::model::re_rolled_action::ReRolledAction;
use crate::enums::{SkillCategory, SkillUsageType, ReRollSource};

pub struct ThinkingMansTroll {
    pub base: Skill,
}

impl ThinkingMansTroll {
    pub fn new() -> Self {
        let mut base = Skill::with_usage_type("Thinking Man's Troll", SkillCategory::Trait, SkillUsageType::OncePerHalf);
        // Java postConstruct: registerRerollSource(ReRolledActions.SINGLE_DIE, ReRollSources.THINKING_MANS_TROLL);
        base.register_reroll_source(ReRolledAction::new("Single Die"), ReRollSource::new("Thinking Man's Troll"));
        Self { base }
    }
}

impl Default for ThinkingMansTroll {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for ThinkingMansTroll {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(ThinkingMansTroll::new().get_name(), "Thinking Man's Troll"); }
    #[test]
    fn category_is_correct() { assert_eq!(ThinkingMansTroll::new().get_category(), SkillCategory::Trait); }
    #[test]
    fn usage_type_is_once_per_half() { assert_eq!(ThinkingMansTroll::new().skill_usage_type, SkillUsageType::OncePerHalf); }
    #[test]
    fn registers_single_die_reroll_source() {
        let skill = ThinkingMansTroll::new();
        let source = skill.base.reroll_sources.get(&ReRolledAction::new("Single Die"));
        assert_eq!(source, Some(&ReRollSource::new("Thinking Man's Troll")));
    }
}
