/// 1:1 translation of com.fumbbl.ffb.skill.bb2025::Defensive.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Defensive {
    pub base: Skill,
}

impl Defensive {
    pub fn new() -> Self {
        let base = Skill::new("Defensive", SkillCategory::Agility);
        Self { base }
    }
}

impl Default for Defensive {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Defensive {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(Defensive::new().get_name(), "Defensive");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(Defensive::new().get_category(), SkillCategory::Agility);
    }
}
