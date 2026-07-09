/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::Accurate.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Accurate {
    pub base: Skill,
}

impl Accurate {
    pub fn new() -> Self {
        let base = Skill::new("Accurate", SkillCategory::Passing);
        Self { base }
    }
}

impl Default for Accurate {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Accurate {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(Accurate::new().get_name(), "Accurate");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(Accurate::new().get_category(), SkillCategory::Passing);
    }
}
