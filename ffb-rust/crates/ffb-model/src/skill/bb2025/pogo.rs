/// 1:1 translation of com.fumbbl.ffb.skill.bb2025::Pogo.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Pogo {
    pub base: Skill,
}

impl Pogo {
    pub fn new() -> Self {
        let base = Skill::new("Pogo", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for Pogo {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Pogo {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(Pogo::new().get_name(), "Pogo");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(Pogo::new().get_category(), SkillCategory::Trait);
    }
}
