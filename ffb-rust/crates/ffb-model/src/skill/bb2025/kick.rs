/// 1:1 translation of com.fumbbl.ffb.skill.bb2025::Kick.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Kick {
    pub base: Skill,
}

impl Kick {
    pub fn new() -> Self {
        let base = Skill::new("Kick", SkillCategory::General);
        Self { base }
    }
}

impl Default for Kick {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Kick {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(Kick::new().get_name(), "Kick");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(Kick::new().get_category(), SkillCategory::General);
    }
}
