/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::Reliable.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Reliable {
    pub base: Skill,
}

impl Reliable {
    pub fn new() -> Self {
        let base = Skill::new("Reliable", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for Reliable {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Reliable {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_reliable() {
        assert_eq!(Reliable::new().get_name(), "Reliable");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(Reliable::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn has_skill_properties_not_null() {
        // Java: assertNotNull(skill.getSkillProperties()); properties() always returns a valid slice.
        let _properties: &'static [&'static str] = crate::enums::SkillId::Reliable.properties();
    }
}
