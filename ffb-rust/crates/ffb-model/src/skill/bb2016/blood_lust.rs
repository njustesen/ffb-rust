/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::BloodLust.
use crate::model::skill::Skill;
use crate::enums::SkillCategory;

pub struct BloodLust {
    pub base: Skill,
}

impl BloodLust {
    pub fn new() -> Self {
        let base = Skill::new("Blood Lust", SkillCategory::Extraordinary);
        Self { base }
    }
}

impl Default for BloodLust {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for BloodLust {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(BloodLust::new().get_name(), "Blood Lust");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(BloodLust::new().get_category(), SkillCategory::Extraordinary);
    }
}
