/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::KeenPlayer.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct KeenPlayer {
    pub base: Skill,
}

impl KeenPlayer {
    pub fn new() -> Self {
        let base = Skill::new("Keen Player", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for KeenPlayer {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for KeenPlayer {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(KeenPlayer::new().get_name(), "Keen Player"); }
    #[test]
    fn category_is_correct() { assert_eq!(KeenPlayer::new().get_category(), SkillCategory::Trait); }
}
