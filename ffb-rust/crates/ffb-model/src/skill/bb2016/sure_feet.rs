/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::SureFeet.
// Java registers `registerRerollSource(ReRolledActions.GO_FOR_IT, ReRollSources.SURE_FEET)`;
// mirrored in the live `SkillId::reroll_sources()` table (Java GO_FOR_IT → "GFI").
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
}

impl Default for SureFeet {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for SureFeet {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
// Mirrors ffb-java/ffb-server/src/test/java/com/fumbbl/ffb/server/skill tests.
mod tests {
    use super::*;

    #[test]
    fn name_is_sure_feet() {
        assert_eq!(SureFeet::new().get_name(), "Sure Feet");
    }

    #[test]
    fn category_is_agility() {
        assert_eq!(SureFeet::new().get_category(), SkillCategory::Agility);
    }

    #[test]
    fn has_sure_feet_reroll_source() {
        // Java: assertNotNull(skill.getRerollSource(ReRolledActions.GO_FOR_IT)) —
        // mirrored against the live SkillId::reroll_sources() table (GO_FOR_IT → "GFI").
        assert!(crate::enums::SkillId::SureFeet.reroll_sources().iter().any(|(a, _)| *a == "GFI"));
    }
}
