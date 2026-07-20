/// 1:1 translation of com.fumbbl.ffb.skill.bb2025::DirtyPlayer.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct DirtyPlayer {
    pub base: Skill,
}

impl DirtyPlayer {
    pub fn new() -> Self {
        let base = Skill::new("Dirty Player", SkillCategory::Devious);
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
    fn name_is_dirty_player() {
        assert_eq!(DirtyPlayer::new().get_name(), "Dirty Player");
    }

    #[test]
    fn category_is_devious() {
        assert_eq!(DirtyPlayer::new().get_category(), SkillCategory::Devious);
    }

    #[test]
    fn has_affects_either_armour_or_injury_on_foul_property() {
        assert!(crate::enums::SkillId::DirtyPlayer.properties().contains(&"affectsEitherArmourOrInjuryOnFoul"));
    }

    #[test]
    fn does_not_have_force_followup_property() {
        assert!(!crate::enums::SkillId::DirtyPlayer.properties().contains(&"forceFollowup"));
    }
}
