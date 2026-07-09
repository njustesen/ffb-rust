/// 1:1 translation of com.fumbbl.ffb.skill.bb2025::Unsteady.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Unsteady {
    pub base: Skill,
}

impl Unsteady {
    pub fn new() -> Self {
        let base = Skill::new("Unsteady", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for Unsteady {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Unsteady {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(Unsteady::new().get_name(), "Unsteady");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(Unsteady::new().get_category(), SkillCategory::Trait);
    }
}
