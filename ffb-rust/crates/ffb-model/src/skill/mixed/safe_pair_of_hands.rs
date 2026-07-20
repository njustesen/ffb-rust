/// 1:1 translation of com.fumbbl.ffb.skill.mixed::SafePairOfHands.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct SafePairOfHands {
    pub base: Skill,
}

impl SafePairOfHands {
    pub fn new() -> Self {
        let base = Skill::new("Safe Pair Of Hands", SkillCategory::Agility);
        Self { base }
    }
}

impl Default for SafePairOfHands {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for SafePairOfHands {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mirrors ffb-java SafePairOfHandsSkillTest.

    #[test]
    fn name_is_safe_pair_of_hands() {
        assert_eq!(SafePairOfHands::new().get_name(), "Safe Pair Of Hands");
    }

    #[test]
    fn category_is_agility() {
        assert_eq!(SafePairOfHands::new().get_category(), SkillCategory::Agility);
    }

    #[test]
    fn has_can_place_ball_when_knocked_down_property() {
        assert!(crate::enums::SkillId::SafePairOfHands
            .properties()
            .contains(&"canPlaceBallWhenKnockedDownOrPlacedProne"));
    }

    #[test]
    fn class_name_is_safe_pair_of_hands() {
        assert_eq!(crate::enums::SkillId::SafePairOfHands.class_name(), "SafePairOfHands");
    }
}
