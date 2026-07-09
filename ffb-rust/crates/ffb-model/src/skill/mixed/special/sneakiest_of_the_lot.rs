/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::SneakiestOfTheLot.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct SneakiestOfTheLot {
    pub base: Skill,
}

impl SneakiestOfTheLot {
    pub fn new() -> Self {
        let base = Skill::new("Sneakiest of the Lot", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for SneakiestOfTheLot {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for SneakiestOfTheLot {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(SneakiestOfTheLot::new().get_name(), "Sneakiest of the Lot"); }
    #[test]
    fn category_is_correct() { assert_eq!(SneakiestOfTheLot::new().get_category(), SkillCategory::Trait); }
}
