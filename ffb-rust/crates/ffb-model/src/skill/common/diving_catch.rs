/// 1:1 translation of com.fumbbl.ffb.skill.common::DivingCatch.
///
/// DEFERRED: Java's postConstruct() also registers a CatchModifier("Diving Catch", -1, REGULAR)
/// with an overridden appliesToContext(...) that checks CatchScatterThrowInMode. This is not
/// translated here because the modifier system in this crate is currently stubbed
/// (see model::skill::skill — CatchModifier etc. are `String` type aliases, no behavior hook
/// exists yet). The `canAttemptCatchInAdjacentSquares` / `addBonusForAccuratePass` properties
/// this skill also registers are tracked in enums::skill_id::SkillId::properties().
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct DivingCatch {
    pub base: Skill,
}

impl DivingCatch {
    pub fn new() -> Self {
        let base = Skill::new("Diving Catch", SkillCategory::Agility);
        Self { base }
    }
}

impl Default for DivingCatch {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for DivingCatch {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(DivingCatch::new().get_name(), "Diving Catch"); }
    #[test]
    fn category_is_correct() { assert_eq!(DivingCatch::new().get_category(), SkillCategory::Agility); }
}
