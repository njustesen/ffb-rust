/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::GhostlyFlames.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct GhostlyFlames {
    pub base: Skill,
}

impl GhostlyFlames {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Ghostly Flames", SkillCategory::Trait, SkillUsageType::OncePerHalf);
        Self { base }
    }
}

impl Default for GhostlyFlames {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for GhostlyFlames {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(GhostlyFlames::new().get_name(), "Ghostly Flames");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(GhostlyFlames::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn usage_type_is_once_per_half() {
        assert_eq!(GhostlyFlames::new().get_skill_usage_type(), SkillUsageType::OncePerHalf);
    }
}
