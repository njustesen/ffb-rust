/// 1:1 translation of com.fumbbl.ffb.skill.mixed::Loner.
use crate::model::player::Player;
use crate::model::skill::skill::Skill;
use crate::model::skill::skill_value_evaluator::SkillValueEvaluator;
use crate::enums::SkillCategory;

pub struct Loner {
    pub base: Skill,
}

impl Loner {
    pub fn new() -> Self {
        let base = Skill::with_default_value("Loner", SkillCategory::Trait, 4);
        Self { base }
    }

    /// Java `getCost(Player<?> player)` override — Loner is always free.
    pub fn get_cost(&self, _player: &Player) -> i32 {
        0
    }

    /// Java `evaluator()` override.
    pub fn evaluator(&self) -> SkillValueEvaluator {
        SkillValueEvaluator::Roll
    }
}

impl Default for Loner {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Loner {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mirrors ffb-java LonerSkillTest.

    #[test]
    fn name_is_loner() {
        assert_eq!(Loner::new().get_name(), "Loner");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(Loner::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn has_has_to_roll_to_use_team_reroll_property() {
        assert!(crate::enums::SkillId::Loner.properties().contains(&"hasToRollToUseTeamReroll"));
    }

    #[test]
    fn class_name_is_loner() {
        assert_eq!(crate::enums::SkillId::Loner.class_name(), "Loner");
    }

    // Additional Rust-side logic tests beyond the Java test class.
    #[test]
    fn default_value_is_four() { assert_eq!(Loner::new().get_default_skill_value(), 4); }
    #[test]
    fn cost_is_zero() {
        let player = Player::default();
        assert_eq!(Loner::new().get_cost(&player), 0);
    }
    #[test]
    fn evaluator_is_roll() {
        assert_eq!(Loner::new().evaluator(), SkillValueEvaluator::Roll);
    }
}
