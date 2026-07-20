/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::RightStuff.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct RightStuff {
    pub base: Skill,
}

impl RightStuff {
    pub fn new() -> Self {
        let base = Skill::new("Right Stuff", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for RightStuff {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for RightStuff {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::SkillId;

    // bb2020/RightStuff is SkillCategory.TRAIT (the bb2016 test's EXTRAORDINARY is bb2016-only).

    #[test]
    fn name_is_right_stuff() {
        assert_eq!(RightStuff::new().get_name(), "Right Stuff");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(RightStuff::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn has_can_be_thrown_if_strength_is_3_or_less_property() {
        // Java bb2016 test asserts canBeThrown; bb2020/RightStuff.postConstruct registers
        // canBeThrownIfStrengthIs3orLess instead.
        assert!(SkillId::RightStuff.properties().contains(&"canBeThrownIfStrengthIs3orLess"));
    }
}
