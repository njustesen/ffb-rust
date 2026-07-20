/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::ArmourIncrease.
// DEFERRED: Java overrides `getCost(Player)` to return 30000. `Skill` has no `get_cost` concept
// in Rust at all yet (no caller anywhere computes skill purchase cost), so this is deferred
// pending that infrastructure rather than adding a dead override here.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct ArmourIncrease {
    pub base: Skill,
}

impl ArmourIncrease {
    pub fn new() -> Self {
        let base = Skill::new("+AV", SkillCategory::StatIncrease);
        Self { base }
    }
}

impl Default for ArmourIncrease {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for ArmourIncrease {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::SkillId;

    #[test]
    fn name_is_plus_av() {
        assert_eq!(ArmourIncrease::new().get_name(), "+AV");
    }

    #[test]
    fn category_is_stat_increase() {
        assert_eq!(ArmourIncrease::new().get_category(), SkillCategory::StatIncrease);
    }

    #[test]
    fn skill_properties_are_not_null() {
        // Java: assertNotNull(skill.getSkillProperties()) — the live Rust property
        // table always yields a slice; assert every entry is a non-empty key.
        assert!(SkillId::ArmourIncrease.properties().iter().all(|p| !p.is_empty()));
    }
}
