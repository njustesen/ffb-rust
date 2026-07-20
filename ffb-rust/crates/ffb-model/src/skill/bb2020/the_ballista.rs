/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::TheBallista.
// Java postConstruct calls registerRerollSource(...) twice (PASS and THROW_TEAM_MATE, both via
// ReRollSources.THE_BALLISTA); mirrored in the live `SkillId::reroll_sources()` table
// (which also carries bb2025's KICK_TEAM_MATE — the table is a cross-edition union).
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct TheBallista {
    pub base: Skill,
}

impl TheBallista {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("The Ballista", SkillCategory::Trait, SkillUsageType::OncePerGame);
        Self { base }
    }
}

impl Default for TheBallista {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for TheBallista {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::SkillId;

    #[test]
    fn name_is_the_ballista() {
        assert_eq!(TheBallista::new().get_name(), "The Ballista");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(TheBallista::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn skill_properties_are_not_null() {
        // Java: assertNotNull(skill.getSkillProperties()); the live Rust property table
        // is SkillId::TheBallista.properties(), which always returns a valid slice.
        assert!(SkillId::TheBallista.properties().iter().all(|p| !p.is_empty()));
    }

    #[test]
    fn usage_type_is_correct() {
        assert_eq!(TheBallista::new().get_skill_usage_type(), SkillUsageType::OncePerGame);
    }

    #[test]
    fn has_the_ballista_reroll_sources() {
        // Java: bb2020/special/TheBallista.postConstruct registers PASS and THROW_TEAM_MATE
        // (both → ReRollSources.THE_BALLISTA, the only priority-2 source); mirrored against
        // the live SkillId::reroll_sources() table.
        let sources = SkillId::TheBallista.reroll_sources();
        assert!(sources.iter().any(|(a, p)| *a == "PASS" && *p == 2));
        assert!(sources.iter().any(|(a, p)| *a == "THROW_TEAM_MATE" && *p == 2));
    }
}
