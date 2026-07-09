/// 1:1 translation of com.fumbbl.ffb.skill.bb2025.special::WorkingInTandem.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct WorkingInTandem {
    pub base: Skill,
}

impl WorkingInTandem {
    pub fn new() -> Self {
        let base = Skill::new("Working in Tandem", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for WorkingInTandem {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for WorkingInTandem {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(WorkingInTandem::new().get_name(), "Working in Tandem");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(WorkingInTandem::new().get_category(), SkillCategory::Trait);
    }
}
