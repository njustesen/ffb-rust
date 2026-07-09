/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::Stunty.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Stunty {
    pub base: Skill,
}

impl Stunty {
    pub fn new() -> Self {
        let base = Skill::new("Stunty", SkillCategory::Extraordinary);
        Self { base }
    }
}

impl Default for Stunty {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Stunty {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(Stunty::new().get_name(), "Stunty");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(Stunty::new().get_category(), SkillCategory::Extraordinary);
    }
}
