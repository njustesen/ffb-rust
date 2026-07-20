/// 1:1 translation of com.fumbbl.ffb.skill.common::Block.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Block {
    pub base: Skill,
}

impl Block {
    pub fn new() -> Self {
        let base = Skill::new("Block", SkillCategory::General);
        Self { base }
    }
}

impl Default for Block {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Block {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_block() {
        assert_eq!(Block::new().get_name(), "Block");
    }

    #[test]
    fn category_is_general() {
        assert_eq!(Block::new().get_category(), SkillCategory::General);
    }

    #[test]
    fn has_prevent_fall_on_both_down_property() {
        // Block must register preventFallOnBothDown so Both Down result does not knock attacker down
        assert!(crate::enums::SkillId::Block.properties().contains(&"preventFallOnBothDown"));
    }

    #[test]
    fn does_not_have_force_followup_property() {
        // Block does not force follow-up (that is Frenzy)
        assert!(!crate::enums::SkillId::Block.properties().contains(&"forceFollowup"));
    }
}
