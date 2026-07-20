/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::KickOffReturn.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct KickOffReturn {
    pub base: Skill,
}

impl KickOffReturn {
    pub fn new() -> Self {
        let base = Skill::new("Kick-Off Return", SkillCategory::General);
        Self { base }
    }
}

impl Default for KickOffReturn {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for KickOffReturn {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::SkillId;

    #[test]
    fn name_is_kick_off_return() {
        assert_eq!(KickOffReturn::new().get_name(), "Kick-Off Return");
    }

    #[test]
    fn category_is_general() {
        assert_eq!(KickOffReturn::new().get_category(), SkillCategory::General);
    }

    #[test]
    fn has_can_move_during_kick_off_scatter_property() {
        assert!(SkillId::KickOffReturn.properties().contains(&"canMoveDuringKickOffScatter"));
    }
}
