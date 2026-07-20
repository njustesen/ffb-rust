/// 1:1 translation of com.fumbbl.ffb.skill.mixed::Kick.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Kick {
    pub base: Skill,
}

impl Kick {
    pub fn new() -> Self {
        let base = Skill::new("Kick", SkillCategory::General);
        Self { base }
    }
}

impl Default for Kick {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Kick {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_kick() {
        assert_eq!(Kick::new().get_name(), "Kick");
    }

    #[test]
    fn category_is_general() {
        assert_eq!(Kick::new().get_category(), SkillCategory::General);
    }

    #[test]
    fn has_can_reduce_kick_distance_property() {
        assert!(crate::enums::SkillId::Kick.properties().contains(&"canReduceKickDistance"));
    }

    // Java: assertNotNull(skill.getSkillProperties()). The live Rust mechanism always
    // returns a valid slice; Kick registers canReduceKickDistance, so the table is non-empty.
    #[test]
    fn has_skill_properties_not_null() {
        assert!(!crate::enums::SkillId::Kick.properties().is_empty());
    }
}
