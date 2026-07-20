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
    fn name_is_plus_ma() {
        assert_eq!(MovementIncrease::new().get_name(), "+MA");
    }

    #[test]
    fn category_is_stat_increase() {
        assert_eq!(MovementIncrease::new().get_category(), SkillCategory::StatIncrease);
    }

    #[test]
    fn skill_properties_are_not_null() {
        // Java asserts getSkillProperties() is not null; the live Rust mechanism
        // always returns a slice (empty here — MovementIncrease registers no NamedProperties).
        assert!(crate::enums::SkillId::MovementIncrease.properties().is_empty());
    }
}
