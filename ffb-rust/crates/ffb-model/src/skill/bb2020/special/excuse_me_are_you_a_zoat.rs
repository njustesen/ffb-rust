/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::ExcuseMeAreYouAZoat.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct ExcuseMeAreYouAZoat {
    pub base: Skill,
}

impl ExcuseMeAreYouAZoat {
    pub fn new() -> Self {
        let base = Skill::new("\"Excuse Me, Are You a Zoat?\"", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for ExcuseMeAreYouAZoat {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for ExcuseMeAreYouAZoat {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(ExcuseMeAreYouAZoat::new().get_name(), "\"Excuse Me, Are You a Zoat?\"");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(ExcuseMeAreYouAZoat::new().get_category(), SkillCategory::Trait);
    }
}
