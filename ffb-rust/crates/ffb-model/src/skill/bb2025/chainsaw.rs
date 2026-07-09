/// 1:1 translation of com.fumbbl.ffb.skill.bb2025::Chainsaw.
use crate::model::skill::skill::Skill;
use crate::enums::{DeclareCondition, SkillCategory};

pub struct Chainsaw {
    pub base: Skill,
}

impl Chainsaw {
    pub fn new() -> Self {
        let mut base = Skill::new("Chainsaw", SkillCategory::Trait);
        base.declare_condition = DeclareCondition::Standing;
        Self { base }
    }
}

impl Default for Chainsaw {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Chainsaw {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(Chainsaw::new().get_name(), "Chainsaw");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(Chainsaw::new().get_category(), SkillCategory::Trait);
    }
}
