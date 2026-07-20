/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::Stab.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Stab {
    pub base: Skill,
}

impl Stab {
    pub fn new() -> Self {
        let base = Skill::new("Stab", SkillCategory::Extraordinary);
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
    use crate::enums::SkillId;

    #[test]
    fn name_is_stab() {
        assert_eq!(Stab::new().get_name(), "Stab");
    }

    #[test]
    fn category_is_extraordinary() {
        assert_eq!(Stab::new().get_category(), SkillCategory::Extraordinary);
    }

    #[test]
    fn has_can_perform_armour_roll_instead_of_block_property() {
        assert!(SkillId::Stab.properties().contains(&"canPerformArmourRollInsteadOfBlock"));
    }
}
