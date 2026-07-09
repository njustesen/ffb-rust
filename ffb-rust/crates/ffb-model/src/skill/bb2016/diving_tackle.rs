/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::DivingTackle.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct DivingTackle {
    pub base: Skill,
}

impl DivingTackle {
    pub fn new() -> Self {
        let base = Skill::new("Diving Tackle", SkillCategory::Agility);
        Self { base }
    }
}

impl Default for DivingTackle {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for DivingTackle {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(DivingTackle::new().get_name(), "Diving Tackle");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(DivingTackle::new().get_category(), SkillCategory::Agility);
    }
}
