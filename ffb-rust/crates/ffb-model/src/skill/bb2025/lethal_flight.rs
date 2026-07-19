/// 1:1 translation of com.fumbbl.ffb.skill.bb2025::LethalFlight.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct LethalFlight {
    pub base: Skill,
}

impl LethalFlight {
    pub fn new() -> Self {
        let base = Skill::new("Lethal Flight", SkillCategory::Devious);
        Self { base }
    }
}

impl Default for LethalFlight {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for LethalFlight {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(LethalFlight::new().get_name(), "Lethal Flight");
    }

    #[test]
    fn category_is_correct() {
        // Java: `super("Lethal Flight", SkillCategory.DEVIOUS)`. A prior translation
        // incorrectly used SkillCategory::Trait here.
        assert_eq!(LethalFlight::new().get_category(), SkillCategory::Devious);
    }
}
