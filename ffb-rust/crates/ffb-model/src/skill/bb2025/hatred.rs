/// 1:1 translation of com.fumbbl.ffb.skill.bb2025::Hatred.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Hatred {
    pub base: Skill,
}

impl Hatred {
    pub fn new() -> Self {
        let base = Skill::new("Hatred", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for Hatred {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Hatred {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_hatred() {
        assert_eq!(Hatred::new().get_name(), "Hatred");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(Hatred::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn has_can_reroll_single_skull_property() {
        assert!(crate::enums::SkillId::Hatred.properties().contains(&"canRerollSingleSkull"));
    }

    #[test]
    fn has_hatred_reroll_source() {
        // Java: bb2025/Hatred.postConstruct registers ReRolledActions.SINGLE_SKULL →
        // ReRollSources.HATRED; mirrored against the live SkillId::reroll_sources() table.
        assert!(crate::enums::SkillId::Hatred.reroll_sources().iter().any(|(a, _)| *a == "SINGLE_SKULL"));
    }
}
