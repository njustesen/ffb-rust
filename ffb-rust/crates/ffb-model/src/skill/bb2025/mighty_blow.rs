/// 1:1 translation of com.fumbbl.ffb.skill.bb2025::MightyBlow.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct MightyBlow {
    pub base: Skill,
}

impl MightyBlow {
    pub fn new() -> Self {
        let base = Skill::with_default_value("Mighty Blow", SkillCategory::Strength, 1);
        Self { base }
    }
}

impl Default for MightyBlow {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for MightyBlow {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(MightyBlow::new().get_name(), "Mighty Blow");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(MightyBlow::new().get_category(), SkillCategory::Strength);
    }
}
