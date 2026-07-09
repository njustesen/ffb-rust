/// 1:1 translation of com.fumbbl.ffb.skill.bb2025::Saboteur.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Saboteur {
    pub base: Skill,
}

impl Saboteur {
    pub fn new() -> Self {
        let base = Skill::new("Saboteur", SkillCategory::Devious);
        Self { base }
    }
}

impl Default for Saboteur {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Saboteur {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(Saboteur::new().get_name(), "Saboteur");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(Saboteur::new().get_category(), SkillCategory::Devious);
    }
}
