/// 1:1 translation of com.fumbbl.ffb.skill.bb2025.special::WoodlandFury.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct WoodlandFury {
    pub base: Skill,
}

impl WoodlandFury {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Woodland Fury", SkillCategory::Trait, SkillUsageType::OncePerGame);
        Self { base }
    }
}

impl Default for WoodlandFury {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for WoodlandFury {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(WoodlandFury::new().get_name(), "Woodland Fury");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(WoodlandFury::new().get_category(), SkillCategory::Trait);
    }
}
