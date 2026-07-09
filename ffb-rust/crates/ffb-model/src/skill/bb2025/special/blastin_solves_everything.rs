/// 1:1 translation of com.fumbbl.ffb.skill.bb2025.special::BlastinSolvesEverything.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct BlastinSolvesEverything {
    pub base: Skill,
}

impl BlastinSolvesEverything {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("\"Blastin' Solves Everything\"", SkillCategory::Trait, SkillUsageType::OncePerHalf);
        Self { base }
    }
}

impl Default for BlastinSolvesEverything {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for BlastinSolvesEverything {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(BlastinSolvesEverything::new().get_name(), "\"Blastin' Solves Everything\"");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(BlastinSolvesEverything::new().get_category(), SkillCategory::Trait);
    }
}
