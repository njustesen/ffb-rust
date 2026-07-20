/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::StrongArm.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct StrongArm {
    pub base: Skill,
}

impl StrongArm {
    pub fn new() -> Self {
        let base = Skill::new("Strong Arm", SkillCategory::Strength);
        Self { base }
    }
}

impl Default for StrongArm {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for StrongArm {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::SkillId;

    #[test]
    fn name_is_strong_arm() {
        assert_eq!(StrongArm::new().get_name(), "Strong Arm");
    }

    #[test]
    fn category_is_strength() {
        assert_eq!(StrongArm::new().get_category(), SkillCategory::Strength);
    }

    #[test]
    fn has_skill_properties_not_null() {
        // Java: assertNotNull(skill.getSkillProperties()) — the live Rust property
        // table always yields a slice; assert every entry is a non-empty key.
        assert!(SkillId::StrongArm.properties().iter().all(|p| !p.is_empty()));
    }
}
