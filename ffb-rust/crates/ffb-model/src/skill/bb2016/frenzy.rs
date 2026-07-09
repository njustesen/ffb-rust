/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::Frenzy.
use crate::model::skill::Skill;
use crate::enums::SkillCategory;

pub struct Frenzy {
    pub base: Skill,
}

impl Frenzy {
    pub fn new() -> Self {
        let base = Skill::new("Frenzy", SkillCategory::General);
        Self { base }
    }
}

impl Default for Frenzy {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Frenzy {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(Frenzy::new().get_name(), "Frenzy");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(Frenzy::new().get_category(), SkillCategory::General);
    }
}
