/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::WildAnimal.
use crate::model::skill::Skill;
use crate::enums::SkillCategory;

pub struct WildAnimal {
    pub base: Skill,
}

impl WildAnimal {
    pub fn new() -> Self {
        let base = Skill::new("Wild Animal", SkillCategory::Extraordinary);
        Self { base }
    }
}

impl Default for WildAnimal {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for WildAnimal {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(WildAnimal::new().get_name(), "Wild Animal");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(WildAnimal::new().get_category(), SkillCategory::Extraordinary);
    }
}
