/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::HitAndRun.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct HitAndRun {
    pub base: Skill,
}

impl HitAndRun {
    pub fn new() -> Self {
        let base = Skill::new("Hit And Run", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for HitAndRun {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for HitAndRun {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(HitAndRun::new().get_name(), "Hit And Run");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(HitAndRun::new().get_category(), SkillCategory::Trait);
    }
}
