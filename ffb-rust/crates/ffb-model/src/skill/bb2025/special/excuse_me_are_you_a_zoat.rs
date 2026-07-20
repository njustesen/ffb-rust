/// 1:1 translation of com.fumbbl.ffb.skill.bb2025.special::ExcuseMeAreYouAZoat.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct ExcuseMeAreYouAZoat {
    pub base: Skill,
}

impl ExcuseMeAreYouAZoat {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("\"Excuse Me, Are You a Zoat?\"", SkillCategory::Trait, SkillUsageType::OncePerGame);
        Self { base }
    }
}

impl Default for ExcuseMeAreYouAZoat {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for ExcuseMeAreYouAZoat {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_contains_zoat() {
        assert!(ExcuseMeAreYouAZoat::new().get_name().contains("Zoat"));
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(ExcuseMeAreYouAZoat::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn has_can_gaze_automatically_three_squares_away_property() {
        assert!(crate::enums::SkillId::ExcuseMeAreYouAZoat.properties().contains(&"canGazeAutomaticallyThreeSquaresAway"));
    }
}
