/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::KickEmWhileTheyReDown.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct KickEmWhileTheyReDown {
    pub base: Skill,
}

impl KickEmWhileTheyReDown {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Kick 'em while they're down!", SkillCategory::Trait, SkillUsageType::OncePerGame);
        Self { base }
    }
}

impl Default for KickEmWhileTheyReDown {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for KickEmWhileTheyReDown {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_kick_em_while_they_re_down() {
        assert_eq!(KickEmWhileTheyReDown::new().get_name(), "Kick 'em while they're down!");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(KickEmWhileTheyReDown::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn has_skill_properties_not_null() {
        // Java: assertNotNull(skill.getSkillProperties()); the live Rust
        // mechanism is SkillId::properties(), whose slice always exists.
        let props: &'static [&'static str] = crate::enums::SkillId::KickEmWhileTheyReDown.properties();
        assert!(props.iter().all(|p| !p.is_empty()));
    }

    #[test]
    fn usage_type_is_once_per_game() {
        assert_eq!(KickEmWhileTheyReDown::new().get_skill_usage_type(), SkillUsageType::OncePerGame);
    }
}
