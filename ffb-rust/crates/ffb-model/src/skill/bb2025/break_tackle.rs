/// 1:1 translation of com.fumbbl.ffb.skill.bb2025::BreakTackle.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct BreakTackle {
    pub base: Skill,
}

impl BreakTackle {
    pub fn new() -> Self {
        let base = Skill::new("Break Tackle", SkillCategory::Strength);
        Self { base }
    }
}

impl Default for BreakTackle {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for BreakTackle {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_break_tackle() {
        assert_eq!(BreakTackle::new().get_name(), "Break Tackle");
    }

    #[test]
    fn category_is_strength() {
        assert_eq!(BreakTackle::new().get_category(), SkillCategory::Strength);
    }

    #[test]
    fn has_skill_properties_not_null() {
        // Java: assertNotNull(skill.getSkillProperties()); the bb2025 Java postConstruct
        // registers no NamedProperties, so the live SkillId table must be empty here.
        assert!(crate::enums::SkillId::BreakTackle.properties().is_empty());
    }
}
