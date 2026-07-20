/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::BreatheFire.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct BreatheFire {
    pub base: Skill,
}

impl BreatheFire {
    pub fn new() -> Self {
        let base = Skill::new("Breathe Fire", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for BreatheFire {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for BreatheFire {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::SkillId;

    #[test]
    fn name_is_breathe_fire() {
        assert_eq!(BreatheFire::new().get_name(), "Breathe Fire");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(BreatheFire::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn has_can_perform_armour_roll_instead_of_block_that_might_fail_property() {
        assert!(SkillId::BreatheFire.properties().contains(&"canPerformArmourRollInsteadOfBlockThatMightFailWithTurnover"));
    }
}
