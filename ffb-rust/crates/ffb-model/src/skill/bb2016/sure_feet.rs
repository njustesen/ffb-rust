/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::SureFeet.
// DEFERRED: Java registers `registerRerollSource(ReRolledActions.GO_FOR_IT, ReRollSources.SURE_FEET)`.
// There is no `ReRollSource::SureFeet` variant and no SkillId-keyed reroll-source lookup table
// anywhere in the workspace (same gap affects bb2016 Monstrous Mouth), so this is deferred
// pending that infrastructure.
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
