/// 1:1 translation of com.fumbbl.ffb.skill.mixed::Accurate.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Accurate {
    pub base: Skill,
}

impl Accurate {
    pub fn new() -> Self {
        let base = Skill::new("Accurate", SkillCategory::Passing);
        Self { base }
    }
}

impl Default for Accurate {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Accurate {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_accurate() {
        assert_eq!(Accurate::new().get_name(), "Accurate");
    }

    #[test]
    fn category_is_passing() {
        assert_eq!(Accurate::new().get_category(), SkillCategory::Passing);
    }

    // Java: assertNotNull(skill.getSkillProperties()). The live Rust mechanism always
    // returns a valid slice; mixed Accurate registers no NamedProperties (only a
    // PassModifier), so the live property table is empty.
    #[test]
    fn has_skill_properties_not_null() {
        assert!(crate::enums::SkillId::Accurate.properties().is_empty());
    }
}
