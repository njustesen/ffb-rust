/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::Incorporeal.
// NOTE: Java (bb2020) postConstruct also calls setStatBasedRollModifierFactory(new StatBasedRollModifierFactory(
// getName(), PlayerStatKey.ST)) to let the player add their Strength to a failed dodge roll. There is no
// per-skill stat-based-roll-modifier registration mechanism in the Rust codebase, so this is not yet wired up.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct Incorporeal {
    pub base: Skill,
}

impl Incorporeal {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Incorporeal", SkillCategory::Trait, SkillUsageType::OncePerGame);
        Self { base }
    }
}

impl Default for Incorporeal {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Incorporeal {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(Incorporeal::new().get_name(), "Incorporeal");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(Incorporeal::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn usage_type_is_correct() {
        assert_eq!(Incorporeal::new().get_skill_usage_type(), SkillUsageType::OncePerGame);
    }

    #[test]
    fn registers_bb2020_named_property() {
        use crate::enums::SkillId;
        // bb2020's Incorporeal registers canAddStrengthToDodge (bb2025's registers canAvoidDodging instead).
        assert!(SkillId::Incorporeal.properties().contains(&"canAddStrengthToDodge"));
    }
}
