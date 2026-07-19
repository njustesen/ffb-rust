/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::PassingIncrease.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct PassingIncrease {
    pub base: Skill,
}

impl PassingIncrease {
    pub fn new() -> Self {
        let base = Skill::new("+PA", SkillCategory::StatIncrease);
        Self { base }
    }

    /// Java `getCost(Player<?> player)` — bb2020 PassingIncrease always costs a flat 30000,
    /// overriding the base Skill.getCost() position/category-based calculation.
    /// NOTE: base Skill.getCost() (position/double-category dependent default) is not yet
    /// wired up in Rust; this returns the fixed override value only.
    pub fn get_cost(&self) -> i32 {
        30000
    }
}

impl Default for PassingIncrease {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for PassingIncrease {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(PassingIncrease::new().get_name(), "+PA");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(PassingIncrease::new().get_category(), SkillCategory::StatIncrease);
    }

    #[test]
    fn cost_is_flat_30000() {
        // Bug: Java PassingIncrease.getCost() always returns 30000, but there was no
        // get_cost() override on the Rust struct at all.
        assert_eq!(PassingIncrease::new().get_cost(), 30000);
    }
}
