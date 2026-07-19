/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::BreakTackle.
// NOTE: Java registers two DodgeModifiers (ST 5+ => -2, ST 4- => -1, both use_strength=true,
// gated on isUseBreakTackle() || hasUnusedSkill()). The live translation of this lives in
// ffb-mechanics::modifiers::dodge_modifier_factory::DodgeModifierFactory::find_skill_modifiers
// (SkillId::BreakTackle, Rules::Bb2020 arm) rather than on this struct, since DodgeModifier
// registration is resolved through that factory, not through Skill::register_modifier (dead code).
// That arm was previously entirely missing (only a Bb2016 arm existed) — fixed as part of this audit.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct BreakTackle {
    pub base: Skill,
}

impl BreakTackle {
    pub fn new() -> Self {
        let base = Skill::new("Break Tackle", SkillCategory::Strength);
        Self { base }
    }
}

impl Default for BreakTackle {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for BreakTackle {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(BreakTackle::new().get_name(), "Break Tackle");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(BreakTackle::new().get_category(), SkillCategory::Strength);
    }
}
