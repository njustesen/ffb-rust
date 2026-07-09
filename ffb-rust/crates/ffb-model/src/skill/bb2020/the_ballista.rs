/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::TheBallista.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct TheBallista {
    pub base: Skill,
}

impl TheBallista {
    pub fn new() -> Self {
        let base = Skill::new("The Ballista", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for TheBallista {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for TheBallista {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(TheBallista::new().get_name(), "The Ballista");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(TheBallista::new().get_category(), SkillCategory::Trait);
    }
}
