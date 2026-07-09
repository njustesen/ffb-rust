/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::LordOfChaos.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct LordOfChaos {
    pub base: Skill,
}

impl LordOfChaos {
    pub fn new() -> Self {
        let base = Skill::new("Lord of Chaos", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for LordOfChaos {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for LordOfChaos {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(LordOfChaos::new().get_name(), "Lord of Chaos");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(LordOfChaos::new().get_category(), SkillCategory::Trait);
    }
}
