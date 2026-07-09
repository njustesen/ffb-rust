/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::RaidingParty.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct RaidingParty {
    pub base: Skill,
}

impl RaidingParty {
    pub fn new() -> Self {
        let base = Skill::new("Raiding Party", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for RaidingParty {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for RaidingParty {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(RaidingParty::new().get_name(), "Raiding Party"); }
    #[test]
    fn category_is_correct() { assert_eq!(RaidingParty::new().get_category(), SkillCategory::Trait); }
}
