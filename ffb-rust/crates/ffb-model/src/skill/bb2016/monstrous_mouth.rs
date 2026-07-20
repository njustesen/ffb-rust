/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::MonstrousMouth.
// Java registers `registerRerollSource(ReRolledActions.CATCH, ReRollSources.MONSTROUS_MOUTH)`;
// mirrored in the live `SkillId::reroll_sources()` table. The
// `cancelsForceOpponentToDropBallOnPushback` property lives in `SkillId::properties()`.
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
    use crate::enums::SkillId;

    #[test]
    fn name_is_monstrous_mouth() {
        assert_eq!(MonstrousMouth::new().get_name(), "Monstrous Mouth");
    }

    #[test]
    fn category_is_extraordinary() {
        assert_eq!(MonstrousMouth::new().get_category(), SkillCategory::Extraordinary);
    }

    #[test]
    fn skill_properties_are_not_null() {
        // Java: assertNotNull(skill.getSkillProperties()) — the live Rust property
        // table always yields a slice; assert every entry is a non-empty key.
        assert!(SkillId::MonstrousMouth.properties().iter().all(|p| !p.is_empty()));
    }

    #[test]
    fn has_monstrous_mouth_reroll_source() {
        // Java: bb2016/MonstrousMouth.postConstruct registers ReRolledActions.CATCH →
        // ReRollSources.MONSTROUS_MOUTH; mirrored against the live SkillId::reroll_sources() table.
        assert!(SkillId::MonstrousMouth.reroll_sources().iter().any(|(a, _)| *a == "CATCH"));
    }
}
