/// 1:1 translation of com.fumbbl.ffb.skill.bb2025.special::BlastIt.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct BlastIt {
    pub base: Skill,
}

impl BlastIt {
    pub fn new() -> Self {
        let base = Skill::new("Blast It!", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for BlastIt {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for BlastIt {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(BlastIt::new().get_name(), "Blast It!");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(BlastIt::new().get_category(), SkillCategory::Trait);
    }
}
