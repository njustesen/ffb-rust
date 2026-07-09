/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::BlindRage.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct BlindRage {
    pub base: Skill,
}

impl BlindRage {
    pub fn new() -> Self {
        let base = Skill::new("Blind Rage", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for BlindRage {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for BlindRage {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(BlindRage::new().get_name(), "Blind Rage"); }
    #[test]
    fn category_is_correct() { assert_eq!(BlindRage::new().get_category(), SkillCategory::Trait); }
}
