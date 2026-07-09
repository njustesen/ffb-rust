/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::Disposable.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Disposable {
    pub base: Skill,
}

impl Disposable {
    pub fn new() -> Self {
        let base = Skill::new("Disposable", SkillCategory::Extraordinary);
        Self { base }
    }
}

impl Default for Disposable {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Disposable {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(Disposable::new().get_name(), "Disposable");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(Disposable::new().get_category(), SkillCategory::Extraordinary);
    }
}
