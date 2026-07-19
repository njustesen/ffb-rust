/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::SureFeet.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct SureFeet {
    pub base: Skill,
}

impl SureFeet {
    pub fn new() -> Self {
        let base = Skill::new("Sure Feet", SkillCategory::Agility);
        Self { base }
    }
    // NOTE: Java postConstruct calls registerRerollSource(ReRolledActions.RUSH, ReRollSources.SURE_FEET);
    // there is no live reroll-source table to mirror that in yet (register_reroll_source is dead code
    // with zero callers, and skill_id.rs has no equivalent static lookup for reroll sources).
}

impl Default for SureFeet {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for SureFeet {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(SureFeet::new().get_name(), "Sure Feet");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(SureFeet::new().get_category(), SkillCategory::Agility);
    }
}
