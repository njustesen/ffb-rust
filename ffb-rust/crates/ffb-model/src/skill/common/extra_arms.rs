/// 1:1 translation of com.fumbbl.ffb.skill.common::ExtraArms.
///
/// DEFERRED: Java's postConstruct() registers a PickupModifier, InterceptionModifier, and
/// CatchModifier (each "Extra Arms", -1, REGULAR). Not translated here because the modifier
/// system in this crate is currently stubbed (see model::skill::skill — PickupModifier,
/// InterceptionModifier, CatchModifier are `String` type aliases with no behavior hook yet).
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct ExtraArms {
    pub base: Skill,
}

impl ExtraArms {
    pub fn new() -> Self {
        let base = Skill::new("Extra Arms", SkillCategory::Mutation);
        Self { base }
    }
}

impl Default for ExtraArms {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for ExtraArms {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(ExtraArms::new().get_name(), "Extra Arms"); }
    #[test]
    fn category_is_correct() { assert_eq!(ExtraArms::new().get_category(), SkillCategory::Mutation); }
}
