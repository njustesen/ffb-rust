/// 1:1 translation of com.fumbbl.ffb.skill.bb2025::Hatred.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Hatred {
    pub base: Skill,
}

impl Hatred {
    pub fn new() -> Self {
        let base = Skill::new("Hatred", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for Hatred {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Hatred {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(Hatred::new().get_name(), "Hatred");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(Hatred::new().get_category(), SkillCategory::Trait);
    }
}
