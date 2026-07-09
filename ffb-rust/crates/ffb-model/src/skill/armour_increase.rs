/// 1:1 translation of com.fumbbl.ffb.skill::ArmourIncrease.
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

    #[test]
    fn name_is_correct() { assert_eq!(ArmourIncrease::new().get_name(), "+AV"); }

    #[test]
    fn category_is_correct() { assert_eq!(ArmourIncrease::new().get_category(), SkillCategory::StatIncrease); }
}
