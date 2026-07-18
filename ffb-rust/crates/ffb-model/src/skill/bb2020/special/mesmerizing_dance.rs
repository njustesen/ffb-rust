/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::MesmerizingDance.
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

    #[test]
    fn name_is_correct() {
        assert_eq!(MesmerizingDance::new().get_name(), "Mesmerizing Dance");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(MesmerizingDance::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn usage_type_is_once_per_game() {
        assert_eq!(MesmerizingDance::new().get_skill_usage_type(), SkillUsageType::OncePerGame);
    }
}
