/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::Swarming.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Swarming {
    pub base: Skill,
}

impl Swarming {
    pub fn new() -> Self {
        let base = Skill::new("Swarming", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for Swarming {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Swarming {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(Swarming::new().get_name(), "Swarming");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(Swarming::new().get_category(), SkillCategory::Trait);
    }
}
