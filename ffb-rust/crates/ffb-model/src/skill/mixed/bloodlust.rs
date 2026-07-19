/// 1:1 translation of com.fumbbl.ffb.skill.mixed::Bloodlust.
use crate::model::skill::skill::Skill;
use crate::model::skill::skill_value_evaluator::SkillValueEvaluator;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct Bloodlust {
    pub base: Skill,
}

impl Bloodlust {
    pub fn new() -> Self {
        let base = Skill::with_all(
            "Bloodlust",
            SkillCategory::Extraordinary,
            2,
            true,
            SkillUsageType::Regular,
        );
        Self { base }
    }

    /// Java `getConfusionMessage()` override.
    pub fn get_confusion_message(&self) -> &'static str {
        "needs to bite a thrall"
    }

    /// Java `evaluator()` override.
    pub fn evaluator(&self) -> SkillValueEvaluator {
        SkillValueEvaluator::Roll
    }
}

impl Default for Bloodlust {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Bloodlust {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(Bloodlust::new().get_name(), "Bloodlust"); }
    #[test]
    fn category_is_correct() { assert_eq!(Bloodlust::new().get_category(), SkillCategory::Extraordinary); }
    #[test]
    fn default_value_is_two() { assert_eq!(Bloodlust::new().get_default_skill_value(), 2); }
    #[test]
    fn is_negative_trait() { assert!(Bloodlust::new().is_negative_trait()); }
    #[test]
    fn confusion_message_overridden() {
        assert_eq!(Bloodlust::new().get_confusion_message(), "needs to bite a thrall");
    }
    #[test]
    fn evaluator_is_roll() {
        assert_eq!(Bloodlust::new().evaluator(), SkillValueEvaluator::Roll);
    }
}
