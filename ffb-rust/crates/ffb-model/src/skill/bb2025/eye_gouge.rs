/// 1:1 translation of com.fumbbl.ffb.skill.bb2025::EyeGouge.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct EyeGouge {
    pub base: Skill,
}

impl EyeGouge {
    pub fn new() -> Self {
        let base = Skill::new("Eye Gouge", SkillCategory::Devious);
        Self { base }
    }
}

impl Default for EyeGouge {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for EyeGouge {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(EyeGouge::new().get_name(), "Eye Gouge");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(EyeGouge::new().get_category(), SkillCategory::Devious);
    }
}
