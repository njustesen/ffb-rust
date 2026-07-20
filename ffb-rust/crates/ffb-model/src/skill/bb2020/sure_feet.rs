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

    // The bb2016 test's has_sure_feet_reroll_source is not mirrored: bb2020/SureFeet.postConstruct
    // only calls registerRerollSource(RUSH, SURE_FEET), and there is no live reroll-source
    // table in Rust to assert against (see the ConsummateProfessional NOTE in skill_id.rs).

    #[test]
    fn name_is_sure_feet() {
        assert_eq!(SureFeet::new().get_name(), "Sure Feet");
    }

    #[test]
    fn category_is_agility() {
        assert_eq!(SureFeet::new().get_category(), SkillCategory::Agility);
    }
}
