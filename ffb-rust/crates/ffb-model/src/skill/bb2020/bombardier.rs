/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::Bombardier.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Bombardier {
    pub base: Skill,
}

impl Bombardier {
    pub fn new() -> Self {
        let base = Skill::new("Bombardier", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for Bombardier {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Bombardier {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::SkillId;

    // bb2020/Bombardier is SkillCategory.TRAIT (the bb2016 test's EXTRAORDINARY is bb2016-only).

    #[test]
    fn name_is_bombardier() {
        assert_eq!(Bombardier::new().get_name(), "Bombardier");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(Bombardier::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn has_enable_throw_bomb_action_property() {
        assert!(SkillId::Bombardier.properties().contains(&"enableThrowBombAction"));
    }
}
