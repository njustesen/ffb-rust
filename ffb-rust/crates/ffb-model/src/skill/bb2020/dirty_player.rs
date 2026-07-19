/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::DirtyPlayer.
// NOTE: Java overrides evaluator() -> SkillValueEvaluator.MODIFIER; Skill::evaluator() has no live
// caller in this workspace so it is not implemented here. The armour/injury modifiers themselves
// ARE live (see ffb-mechanics::modifiers::{armor_modifier_factory, injury_modifier_factory},
// SkillId::DirtyPlayer arms), but use a flat StaticArmourModifier(+1)/StaticInjuryModifierAttacker(+1)
// for all editions rather than Java bb2020/bb2025's VariableArmourModifier/VariableInjuryModifierAttacker
// (whose value is `attacker.getSkillIntValue(DirtyPlayer)`) — Player::get_skill_int_value is
// currently stubbed to always return 0, so a literal translation would zero out the bonus entirely;
// the flat +1 is a deliberate stand-in until that stub is implemented.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct DirtyPlayer {
    pub base: Skill,
}

impl DirtyPlayer {
    pub fn new() -> Self {
        let base = Skill::with_default_value("Dirty Player", SkillCategory::General, 1);
        Self { base }
    }
}

impl Default for DirtyPlayer {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for DirtyPlayer {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(DirtyPlayer::new().get_name(), "Dirty Player");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(DirtyPlayer::new().get_category(), SkillCategory::General);
    }

    #[test]
    fn default_skill_value_is_correct() {
        assert_eq!(DirtyPlayer::new().get_default_skill_value(), 1);
    }
}
