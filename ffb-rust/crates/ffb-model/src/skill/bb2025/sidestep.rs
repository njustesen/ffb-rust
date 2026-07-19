/// 1:1 translation of com.fumbbl.ffb.skill.bb2025::Sidestep.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Sidestep {
    pub base: Skill,
}

impl Sidestep {
    pub fn new() -> Self {
        let base = Skill::new("Sidestep", SkillCategory::Agility);
        Self { base }
    }

    /// Java `getSkillUseDescription()` override — shadows `Skill::get_skill_use_description`
    /// (which returns `None`) via inherent method resolution taking priority over `Deref`.
    pub fn get_skill_use_description(&self) -> Option<Vec<String>> {
        Some(vec![
            "Using Sidestep will allow you to chose the square you are pushed to.".to_string(),
        ])
    }
}

impl Default for Sidestep {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Sidestep {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(Sidestep::new().get_name(), "Sidestep");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(Sidestep::new().get_category(), SkillCategory::Agility);
    }

    #[test]
    fn skill_use_description_matches_java_override() {
        // Java overrides getSkillUseDescription(); base Skill::get_skill_use_description
        // returns None, which is what the initial translation left in place (missing override).
        let desc = Sidestep::new().get_skill_use_description();
        assert_eq!(
            desc,
            Some(vec!["Using Sidestep will allow you to chose the square you are pushed to.".to_string()])
        );
    }
}
