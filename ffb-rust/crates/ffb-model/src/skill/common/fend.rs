/// 1:1 translation of com.fumbbl.ffb.skill.common::Fend.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Fend {
    pub base: Skill,
}

impl Fend {
    pub fn new() -> Self {
        let base = Skill::new("Fend", SkillCategory::General);
        Self { base }
    }
}

impl Default for Fend {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Fend {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_fend() {
        assert_eq!(Fend::new().get_name(), "Fend");
    }

    #[test]
    fn category_is_general() {
        assert_eq!(Fend::new().get_category(), SkillCategory::General);
    }

    #[test]
    fn has_prevent_opponent_following_up_property() {
        assert!(crate::enums::SkillId::Fend.properties().contains(&"preventOpponentFollowingUp"));
    }
}
