/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::MesmerizingDance.
// NOTE: Java postConstruct calls registerRerollSource(ReRolledActions.HYPNOTIC_GAZE, ReRollSources.
// MESMERIZING_DANCE). There is no live reroll-source lookup table in the Rust codebase to mirror this into
// (Skill::register_reroll_source is dead code), so this is left as a gap pending that infrastructure.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct MesmerizingDance {
    pub base: Skill,
}

impl MesmerizingDance {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Mesmerizing Dance", SkillCategory::Trait, SkillUsageType::OncePerGame);
        Self { base }
    }
}

impl Default for MesmerizingDance {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for MesmerizingDance {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::SkillId;

    #[test]
    fn name_is_mesmerizing_dance() {
        assert_eq!(MesmerizingDance::new().get_name(), "Mesmerizing Dance");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(MesmerizingDance::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn skill_properties_are_not_null() {
        // Java: assertNotNull(skill.getSkillProperties()); the live Rust property table
        // is SkillId::MesmerizingDance.properties(), which always returns a valid slice.
        assert!(SkillId::MesmerizingDance.properties().iter().all(|p| !p.is_empty()));
    }

    #[test]
    fn usage_type_is_correct() {
        assert_eq!(MesmerizingDance::new().get_skill_usage_type(), SkillUsageType::OncePerGame);
    }
}
