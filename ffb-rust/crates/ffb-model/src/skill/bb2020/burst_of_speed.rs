/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::BurstOfSpeed.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct BurstOfSpeed {
    pub base: Skill,
}

impl BurstOfSpeed {
    pub fn new() -> Self {
        let base = Skill::new("Burst of Speed", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for BurstOfSpeed {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for BurstOfSpeed {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(BurstOfSpeed::new().get_name(), "Burst of Speed");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(BurstOfSpeed::new().get_category(), SkillCategory::Trait);
    }
}
