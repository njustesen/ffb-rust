/// 1:1 translation of com.fumbbl.ffb.skill.bb2025::Bullseye.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Bullseye {
    pub base: Skill,
}

impl Bullseye {
    pub fn new() -> Self {
        let base = Skill::new("Bullseye", SkillCategory::Strength);
        Self { base }
    }
}

impl Default for Bullseye {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Bullseye {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(Bullseye::new().get_name(), "Bullseye");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(Bullseye::new().get_category(), SkillCategory::Strength);
    }
}
