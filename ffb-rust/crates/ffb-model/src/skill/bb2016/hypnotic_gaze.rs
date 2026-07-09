/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::HypnoticGaze.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct HypnoticGaze {
    pub base: Skill,
}

impl HypnoticGaze {
    pub fn new() -> Self {
        let base = Skill::new("Hypnotic Gaze", SkillCategory::Extraordinary);
        Self { base }
    }
}

impl Default for HypnoticGaze {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for HypnoticGaze {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(HypnoticGaze::new().get_name(), "Hypnotic Gaze");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(HypnoticGaze::new().get_category(), SkillCategory::Extraordinary);
    }
}
