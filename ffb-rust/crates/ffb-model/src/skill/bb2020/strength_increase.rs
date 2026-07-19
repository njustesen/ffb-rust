/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::StrengthIncrease.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct StrengthIncrease {
    pub base: Skill,
}

impl StrengthIncrease {
    pub fn new() -> Self {
        let base = Skill::new("+ST", SkillCategory::StatIncrease);
        Self { base }
    }

    /// Java `getCost(Player<?> player)` — bb2020 StrengthIncrease always costs a flat 80000,
    /// overriding the base Skill.getCost() position/category-based calculation.
    /// NOTE: base Skill.getCost() (position/double-category dependent default) is not yet
    /// wired up in Rust; this returns the fixed override value only.
    pub fn get_cost(&self) -> i32 {
        80000
    }
}

impl Default for StrengthIncrease {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for StrengthIncrease {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(StrengthIncrease::new().get_name(), "+ST");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(StrengthIncrease::new().get_category(), SkillCategory::StatIncrease);
    }

    #[test]
    fn cost_is_flat_80000() {
        // Bug: Java bb2020 StrengthIncrease.getCost() always returns 80000, but there was
        // no get_cost() override on the Rust struct at all.
        assert_eq!(StrengthIncrease::new().get_cost(), 80000);
    }
}
