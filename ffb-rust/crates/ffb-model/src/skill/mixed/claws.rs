/// 1:1 translation of com.fumbbl.ffb.skill.mixed::Claws.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Claws {
    pub base: Skill,
}

impl Claws {
    pub fn new() -> Self {
        let base = Skill::new("Claws", SkillCategory::Mutation);
        Self { base }
    }
}

impl Default for Claws {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Claws {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_claws() {
        assert_eq!(Claws::new().get_name(), "Claws");
    }

    #[test]
    fn category_is_mutation() {
        assert_eq!(Claws::new().get_category(), SkillCategory::Mutation);
    }

    // The mixed Claws class shares SkillId::Claw with bb2016's Claw.
    #[test]
    fn has_reduces_armour_to_fixed_value_property() {
        assert!(crate::enums::SkillId::Claw.properties().contains(&"reducesArmourToFixedValue"));
    }

    #[test]
    fn class_name_is_claws() {
        assert_eq!(
            crate::enums::SkillId::from_class_name("Claws"),
            Some(crate::enums::SkillId::Claw)
        );
    }
}
