/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::Timmmber.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Timmmber {
    pub base: Skill,
}

impl Timmmber {
    pub fn new() -> Self {
        let base = Skill::new("Timmm-ber!", SkillCategory::Extraordinary);
        Self { base }
    }
}

impl Default for Timmmber {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Timmmber {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(Timmmber::new().get_name(), "Timmm-ber!");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(Timmmber::new().get_category(), SkillCategory::Extraordinary);
    }
}
