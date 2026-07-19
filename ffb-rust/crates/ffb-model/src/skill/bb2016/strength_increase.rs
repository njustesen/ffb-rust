/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::StrengthIncrease.
// DEFERRED: Java overrides `getCost(Player)` to return 50000; `Skill` has no `get_cost` concept
// in Rust yet (no caller computes skill purchase cost), so this is deferred pending that
// infrastructure.
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
}
