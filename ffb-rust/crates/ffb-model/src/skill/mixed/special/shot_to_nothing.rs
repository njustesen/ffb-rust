/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::ShotToNothing.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct ShotToNothing {
    pub base: Skill,
}

impl ShotToNothing {
    pub fn new() -> Self {
        let base = Skill::new("Shot to Nothing", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for ShotToNothing {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for ShotToNothing {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(ShotToNothing::new().get_name(), "Shot to Nothing"); }
    #[test]
    fn category_is_correct() { assert_eq!(ShotToNothing::new().get_category(), SkillCategory::Trait); }
}
