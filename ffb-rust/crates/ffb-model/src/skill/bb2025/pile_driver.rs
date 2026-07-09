/// 1:1 translation of com.fumbbl.ffb.skill.bb2025::PileDriver.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct PileDriver {
    pub base: Skill,
}

impl PileDriver {
    pub fn new() -> Self {
        let base = Skill::new("Pile Driver", SkillCategory::Devious);
        Self { base }
    }
}

impl Default for PileDriver {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for PileDriver {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(PileDriver::new().get_name(), "Pile Driver");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(PileDriver::new().get_category(), SkillCategory::Devious);
    }
}
