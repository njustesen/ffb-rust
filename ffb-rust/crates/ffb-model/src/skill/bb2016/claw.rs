/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::Claw.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Claw {
    pub base: Skill,
}

impl Claw {
    pub fn new() -> Self {
        let base = Skill::new("Claw", SkillCategory::Mutation);
        Self { base }
    }
}

impl Default for Claw {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Claw {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::SkillId;

    #[test]
    fn name_is_claw() {
        assert_eq!(Claw::new().get_name(), "Claw");
    }

    #[test]
    fn category_is_mutation() {
        assert_eq!(Claw::new().get_category(), SkillCategory::Mutation);
    }

    #[test]
    fn has_reduces_armour_to_fixed_value_property() {
        assert!(SkillId::Claw.properties().contains(&"reducesArmourToFixedValue"));
    }
}
