/// 1:1 translation of com.fumbbl.ffb.skill.bb2025::CloudBurster.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct CloudBurster {
    pub base: Skill,
}

impl CloudBurster {
    pub fn new() -> Self {
        let base = Skill::new("Cloud Burster", SkillCategory::Passing);
        Self { base }
    }
}

impl Default for CloudBurster {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for CloudBurster {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_cloud_burster() {
        assert_eq!(CloudBurster::new().get_name(), "Cloud Burster");
    }

    #[test]
    fn category_is_passing() {
        assert_eq!(CloudBurster::new().get_category(), SkillCategory::Passing);
    }

    #[test]
    fn has_passes_are_not_intercepted_property() {
        assert!(crate::enums::SkillId::CloudBurster.properties().contains(&"passesAreNotIntercepted"));
    }
}
