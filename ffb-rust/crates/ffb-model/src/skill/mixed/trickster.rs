/// 1:1 translation of com.fumbbl.ffb.skill.mixed::Trickster.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Trickster {
    pub base: Skill,
}

impl Trickster {
    pub fn new() -> Self {
        let base = Skill::new("Trickster", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for Trickster {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Trickster {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(Trickster::new().get_name(), "Trickster"); }
    #[test]
    fn category_is_correct() { assert_eq!(Trickster::new().get_category(), SkillCategory::Trait); }
}
