/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::UnstoppableMomentum.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct UnstoppableMomentum {
    pub base: Skill,
}

impl UnstoppableMomentum {
    pub fn new() -> Self {
        let base = Skill::new("Unstoppable Momentum", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for UnstoppableMomentum {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for UnstoppableMomentum {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(UnstoppableMomentum::new().get_name(), "Unstoppable Momentum"); }
    #[test]
    fn category_is_correct() { assert_eq!(UnstoppableMomentum::new().get_category(), SkillCategory::Trait); }
}
