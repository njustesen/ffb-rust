/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::MonstrousMouth.
// DEFERRED: Java registers `registerRerollSource(ReRolledActions.CATCH, ReRollSources.MONSTROUS_MOUTH)`.
// The `cancelsForceOpponentToDropBallOnPushback` property is now in `SkillId::properties()`
// (Phase AJ fix), but there is no `ReRollSource::MonstrousMouth` variant and no SkillId-keyed
// reroll-source lookup table anywhere in the workspace (same gap affects bb2016 Sure Feet), so
// the reroll-on-failed-catch grant is deferred pending that infrastructure.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct MonstrousMouth {
    pub base: Skill,
}

impl MonstrousMouth {
    pub fn new() -> Self {
        let base = Skill::new("Monstrous Mouth", SkillCategory::Extraordinary);
        Self { base }
    }
}

impl Default for MonstrousMouth {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for MonstrousMouth {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(MonstrousMouth::new().get_name(), "Monstrous Mouth");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(MonstrousMouth::new().get_category(), SkillCategory::Extraordinary);
    }
}
