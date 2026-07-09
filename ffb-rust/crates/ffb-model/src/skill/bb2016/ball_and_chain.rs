/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::BallAndChain.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct BallAndChain {
    pub base: Skill,
}

impl BallAndChain {
    pub fn new() -> Self {
        let base = Skill::new("Ball and Chain", SkillCategory::Extraordinary);
        Self { base }
    }
}

impl Default for BallAndChain {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for BallAndChain {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(BallAndChain::new().get_name(), "Ball and Chain");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(BallAndChain::new().get_category(), SkillCategory::Extraordinary);
    }
}
