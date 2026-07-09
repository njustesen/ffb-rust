/// 1:1 translation of com.fumbbl.ffb.skill.bb2025::Fumblerooski.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Fumblerooski {
    pub base: Skill,
}

impl Fumblerooski {
    pub fn new() -> Self {
        let base = Skill::new("Fumblerooski", SkillCategory::Devious);
        Self { base }
    }
}

impl Default for Fumblerooski {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Fumblerooski {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(Fumblerooski::new().get_name(), "Fumblerooski");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(Fumblerooski::new().get_category(), SkillCategory::Devious);
    }
}
