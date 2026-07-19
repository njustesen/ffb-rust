/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::DwarfenScourge.
// NOTE: Java postConstruct registers a VariableArmourModifier and a VariableInjuryModifierAttacker (both
// +1, or +2 vs dwarf defenders), but both override appliesToContext() to unconditionally return false — a
// no-op in Java too. No per-skill dynamic modifier registration mechanism exists in Rust regardless.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct DwarfenScourge {
    pub base: Skill,
}

impl DwarfenScourge {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Dwarfen Scourge", SkillCategory::Trait, SkillUsageType::OncePerGame);
        Self { base }
    }
}

impl Default for DwarfenScourge {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for DwarfenScourge {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(DwarfenScourge::new().get_name(), "Dwarfen Scourge");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(DwarfenScourge::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn usage_type_is_correct() {
        assert_eq!(DwarfenScourge::new().get_skill_usage_type(), SkillUsageType::OncePerGame);
    }
}
