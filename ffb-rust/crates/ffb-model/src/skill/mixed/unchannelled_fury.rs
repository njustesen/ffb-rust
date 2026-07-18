/// 1:1 translation of com.fumbbl.ffb.skill.mixed::UnchannelledFury.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct UnchannelledFury {
    pub base: Skill,
}

impl UnchannelledFury {
    pub fn new() -> Self {
        let base = Skill::as_negative_trait("Unchannelled Fury", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for UnchannelledFury {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for UnchannelledFury {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(UnchannelledFury::new().get_name(), "Unchannelled Fury"); }
    #[test]
    fn category_is_correct() { assert_eq!(UnchannelledFury::new().get_category(), SkillCategory::Trait); }
    #[test]
    fn is_negative_trait() { assert!(UnchannelledFury::new().is_negative_trait()); }
}
