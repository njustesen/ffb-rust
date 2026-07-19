/// 1:1 translation of com.fumbbl.ffb.skill.common::MovementIncrease.
// NOTE: Java's `getCost(Player)` override (returns 30000) is not translated here.
// The Rust `Skill` struct has no `get_cost`/cost concept anywhere in the skill
// module yet (no infra exists for per-skill purchase cost), so this is deferred
// until that infrastructure is ported.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct MovementIncrease {
    pub base: Skill,
}

impl MovementIncrease {
    pub fn new() -> Self {
        let base = Skill::new("+MA", SkillCategory::StatIncrease);
        Self { base }
    }
}

impl Default for MovementIncrease {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for MovementIncrease {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(MovementIncrease::new().get_name(), "+MA"); }
    #[test]
    fn category_is_correct() { assert_eq!(MovementIncrease::new().get_category(), SkillCategory::StatIncrease); }
}
