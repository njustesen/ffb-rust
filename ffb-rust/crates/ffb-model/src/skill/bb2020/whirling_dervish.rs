/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::WhirlingDervish.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct WhirlingDervish {
    pub base: Skill,
}

impl WhirlingDervish {
    pub fn new() -> Self {
        let base = Skill::new("Whirling Dervish", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for WhirlingDervish {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for WhirlingDervish {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(WhirlingDervish::new().get_name(), "Whirling Dervish");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(WhirlingDervish::new().get_category(), SkillCategory::Trait);
    }
}
