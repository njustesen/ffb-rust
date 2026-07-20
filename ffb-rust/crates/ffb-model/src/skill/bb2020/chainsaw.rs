/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::Chainsaw.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Chainsaw {
    pub base: Skill,
}

impl Chainsaw {
    pub fn new() -> Self {
        let base = Skill::new("Chainsaw", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for Chainsaw {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Chainsaw {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::SkillId;

    // bb2020/Chainsaw is SkillCategory.TRAIT (the bb2016 test's EXTRAORDINARY is bb2016-only).

    #[test]
    fn name_is_chainsaw() {
        assert_eq!(Chainsaw::new().get_name(), "Chainsaw");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(Chainsaw::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn has_blocks_like_chainsaw_property() {
        // Java bb2016 test asserts makesStrengthTestObsolete, which only the bb2016 edition
        // registers; bb2020/Chainsaw.postConstruct registers blocksLikeChainsaw instead.
        assert!(SkillId::Chainsaw.properties().contains(&"blocksLikeChainsaw"));
    }
}
