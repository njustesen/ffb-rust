/// 1:1 translation of com.fumbbl.ffb.skill.mixed::MultipleBlock.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct MultipleBlock {
    pub base: Skill,
}

impl MultipleBlock {
    pub fn new() -> Self {
        let base = Skill::new("Multiple Block", SkillCategory::Strength);
        Self { base }
    }
}

impl Default for MultipleBlock {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for MultipleBlock {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mirrors ffb-java MultipleBlockSkillTest (written against the bb2016 class);
    // property assertions adapted to the mixed edition's postConstruct, which
    // registers canBlockTwoAtOnce (canBlockMoreThanOnce is bb2016-only).
    // The Java `is_bb2016_edition` test is skipped (edition annotations are
    // covered elsewhere).

    #[test]
    fn name_is_multiple_block() {
        assert_eq!(MultipleBlock::new().get_name(), "Multiple Block");
    }

    #[test]
    fn category_is_strength() {
        assert_eq!(MultipleBlock::new().get_category(), SkillCategory::Strength);
    }

    #[test]
    fn has_can_block_two_at_once_property() {
        assert!(crate::enums::SkillId::MultipleBlock.properties().contains(&"canBlockTwoAtOnce"));
    }
}
