/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::NervesOfSteel.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct NervesOfSteel {
    pub base: Skill,
}

impl NervesOfSteel {
    pub fn new() -> Self {
        let base = Skill::new("Nerves of Steel", SkillCategory::Passing);
        Self { base }
    }
}

impl Default for NervesOfSteel {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for NervesOfSteel {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(NervesOfSteel::new().get_name(), "Nerves of Steel");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(NervesOfSteel::new().get_category(), SkillCategory::Passing);
    }
}
