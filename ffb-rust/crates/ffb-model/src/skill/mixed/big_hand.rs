/// 1:1 translation of com.fumbbl.ffb.skill.mixed::BigHand.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct BigHand {
    pub base: Skill,
}

impl BigHand {
    pub fn new() -> Self {
        let base = Skill::new("Big Hand", SkillCategory::Mutation);
        Self { base }
    }
}

impl Default for BigHand {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for BigHand {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_big_hand() {
        assert_eq!(BigHand::new().get_name(), "Big Hand");
    }

    #[test]
    fn category_is_mutation() {
        assert_eq!(BigHand::new().get_category(), SkillCategory::Mutation);
    }

    #[test]
    fn has_ignore_tacklezones_when_picking_up_property() {
        assert!(crate::enums::SkillId::BigHand.properties().contains(&"ignoreTacklezonesWhenPickingUp"));
    }

    #[test]
    fn class_name_is_big_hand() {
        assert_eq!(crate::enums::SkillId::BigHand.class_name(), "BigHand");
    }
}
