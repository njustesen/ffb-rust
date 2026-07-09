/// 1:1 translation of com.fumbbl.ffb.skill.bb2025::Sidestep.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Sidestep {
    pub base: Skill,
}

impl Sidestep {
    pub fn new() -> Self {
        let base = Skill::new("Sidestep", SkillCategory::Agility);
        Self { base }
    }
}

impl Default for Sidestep {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Sidestep {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(Sidestep::new().get_name(), "Sidestep");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(Sidestep::new().get_category(), SkillCategory::Agility);
    }
}
