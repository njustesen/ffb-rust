/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::Incorporeal.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Incorporeal {
    pub base: Skill,
}

impl Incorporeal {
    pub fn new() -> Self {
        let base = Skill::new("Incorporeal", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for Incorporeal {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Incorporeal {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(Incorporeal::new().get_name(), "Incorporeal");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(Incorporeal::new().get_category(), SkillCategory::Trait);
    }
}
