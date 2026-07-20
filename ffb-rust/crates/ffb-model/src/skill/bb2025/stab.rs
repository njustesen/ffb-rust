/// 1:1 translation of com.fumbbl.ffb.skill.bb2025::Stab.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, DeclareCondition};

pub struct Stab {
    pub base: Skill,
}

impl Stab {
    pub fn new() -> Self {
        let mut base = Skill::new("Stab", SkillCategory::Trait);
        base.set_declare_condition(DeclareCondition::Standing);
        Self { base }
    }
}

impl Default for Stab {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Stab {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_stab() {
        assert_eq!(Stab::new().get_name(), "Stab");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(Stab::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn has_can_perform_armour_roll_instead_of_block_property() {
        assert!(crate::enums::SkillId::Stab.properties().contains(&"canPerformArmourRollInsteadOfBlock"));
    }

    #[test]
    fn declare_condition_is_standing() {
        // Java Stab() constructor calls setDeclareCondition(DeclareCondition.STANDING).
        assert_eq!(Stab::new().get_declare_condition(), DeclareCondition::Standing);
    }
}
