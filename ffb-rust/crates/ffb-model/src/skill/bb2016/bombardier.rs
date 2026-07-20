/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::Bombardier.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Bombardier {
    pub base: Skill,
}

impl Bombardier {
    pub fn new() -> Self {
        let base = Skill::new("Bombardier", SkillCategory::Extraordinary);
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

    #[test]
    fn name_is_bombardier() {
        assert_eq!(Bombardier::new().get_name(), "Bombardier");
    }

    #[test]
    fn category_is_extraordinary() {
        assert_eq!(Bombardier::new().get_category(), SkillCategory::Extraordinary);
    }

    #[test]
    fn has_enable_throw_bomb_action_property() {
        assert!(SkillId::Bombardier.properties().contains(&"enableThrowBombAction"));
    }
}
