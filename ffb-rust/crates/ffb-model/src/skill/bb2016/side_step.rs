/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::SideStep.
// DEFERRED: Java overrides `getSkillUseDescription()` with a one-line UI hint string. Rust
// `Skill::get_skill_use_description()` exists but has no consumer anywhere in the workspace yet,
// so an override here would be dead code. Deferred pending that UI-hint infrastructure.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct SideStep {
    pub base: Skill,
}

impl SideStep {
    pub fn new() -> Self {
        let base = Skill::new("Side Step", SkillCategory::Agility);
        Self { base }
    }
}

impl Default for SideStep {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for SideStep {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(SideStep::new().get_name(), "Side Step");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(SideStep::new().get_category(), SkillCategory::Agility);
    }
}
