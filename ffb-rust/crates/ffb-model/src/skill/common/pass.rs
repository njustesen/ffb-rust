/// 1:1 translation of com.fumbbl.ffb.skill.common::Pass.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, ReRollSource};
use crate::model::re_rolled_action::ReRolledAction;

pub struct Pass {
    pub base: Skill,
}

impl Pass {
    pub fn new() -> Self {
        let mut base = Skill::new("Pass", SkillCategory::Passing);
        // Java postConstruct(): registerRerollSource(ReRolledActions.PASS, ReRollSources.PASS);
        base.register_reroll_source(
            ReRolledAction::new("com.fumbbl.ffb.skill.common.Pass"),
            ReRollSource::new("Pass"),
        );
        Self { base }
    }
}

impl Default for Pass {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Pass {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(Pass::new().get_name(), "Pass"); }
    #[test]
    fn category_is_correct() { assert_eq!(Pass::new().get_category(), SkillCategory::Passing); }
    #[test]
    fn registers_pass_reroll_source() {
        // Java Pass.postConstruct() registers a reroll source for the PASS action;
        // this would have failed before the fix since no reroll source was registered.
        let p = Pass::new();
        let action = ReRolledAction::new("com.fumbbl.ffb.skill.common.Pass");
        let source = p.get_reroll_source(&action);
        assert!(source.is_some());
        assert_eq!(source.unwrap().name, "Pass");
    }
}
