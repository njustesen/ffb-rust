/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::Animosity.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Animosity {
    pub base: Skill,
}

impl Animosity {
    pub fn new() -> Self {
        let base = Skill::new("Animosity", SkillCategory::Extraordinary);
        Self { base }
    }
}

impl Default for Animosity {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Animosity {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
// Mirrors ffb-java/ffb-server/src/test/java/com/fumbbl/ffb/server/skill tests.
// Java test targets bb2025 (category TRAIT); bb2016/Animosity.java is EXTRAORDINARY.
mod tests {
    use super::*;
    use crate::enums::SkillId;

    #[test]
    fn name_is_animosity() {
        assert_eq!(Animosity::new().get_name(), "Animosity");
    }

    #[test]
    fn category_is_extraordinary() {
        assert_eq!(Animosity::new().get_category(), SkillCategory::Extraordinary);
    }

    #[test]
    fn has_has_to_roll_to_pass_ball_on_property() {
        assert!(SkillId::Animosity.properties().contains(&"hasToRollToPassBallOn"));
    }
}
