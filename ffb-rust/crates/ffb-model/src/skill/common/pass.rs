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
    fn name_is_pass() {
        assert_eq!(Pass::new().get_name(), "Pass");
    }

    #[test]
    fn category_is_passing() {
        assert_eq!(Pass::new().get_category(), SkillCategory::Passing);
    }

    #[test]
    fn skill_properties_are_not_null() {
        // Java asserts getSkillProperties() is not null; the live Rust mechanism
        // always returns a slice (empty here — Pass registers only a reroll source, no NamedProperties).
        assert!(crate::enums::SkillId::Pass.properties().is_empty());
    }

    #[test]
    fn has_pass_reroll_source() {
        // Java: assertNotNull(skill.getRerollSource(ReRolledActions.PASS)) —
        // mirrored against the live SkillId::reroll_sources() table (the
        // per-struct register_reroll_source map is inert).
        assert!(crate::enums::SkillId::Pass.reroll_sources().iter().any(|(a, _)| *a == "PASS"));
    }
}
