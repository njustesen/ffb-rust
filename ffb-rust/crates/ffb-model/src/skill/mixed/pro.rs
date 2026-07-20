/// 1:1 translation of com.fumbbl.ffb.skill.mixed::Pro.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Pro {
    pub base: Skill,
}

impl Pro {
    pub fn new() -> Self {
        let base = Skill::new("Pro", SkillCategory::General);
        Self { base }
    }
}

impl Default for Pro {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Pro {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mirrors ffb-java ProSkillTest.

    #[test]
    fn name_is_pro() {
        assert_eq!(Pro::new().get_name(), "Pro");
    }

    #[test]
    fn category_is_general() {
        assert_eq!(Pro::new().get_category(), SkillCategory::General);
    }

    #[test]
    fn has_can_reroll_once_per_turn_property() {
        assert!(crate::enums::SkillId::Pro.properties().contains(&"canRerollOncePerTurn"));
    }

    #[test]
    fn class_name_is_pro() {
        assert_eq!(crate::enums::SkillId::Pro.class_name(), "Pro");
    }
}
