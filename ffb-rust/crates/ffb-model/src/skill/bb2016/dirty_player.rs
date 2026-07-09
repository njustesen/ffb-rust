/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::DirtyPlayer.
use crate::model::skill::Skill;
use crate::enums::SkillCategory;

pub struct DirtyPlayer {
    pub base: Skill,
}

impl DirtyPlayer {
    pub fn new() -> Self {
        let base = Skill::new("Dirty Player", SkillCategory::General);
        Self { base }
    }
}

impl Default for DirtyPlayer {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for DirtyPlayer {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(DirtyPlayer::new().get_name(), "Dirty Player");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(DirtyPlayer::new().get_category(), SkillCategory::General);
    }
}
