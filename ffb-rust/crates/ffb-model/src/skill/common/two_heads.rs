/// 1:1 translation of com.fumbbl.ffb.skill.common::TwoHeads.
// NOTE: Java's postConstruct() does:
//   registerModifier(new DodgeModifier("Two Heads", -1, ModifierType.REGULAR));
// `DodgeModifier` is stubbed as `String` in the Rust `Skill` struct (no
// modifier subsystem ported yet, no numeric value/ModifierType fields exist),
// so the actual -1 dodge modifier cannot be represented/registered with the
// current infra. Deferred until the dodge modifier subsystem is ported.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct TwoHeads {
    pub base: Skill,
}

impl TwoHeads {
    pub fn new() -> Self {
        let base = Skill::new("Two Heads", SkillCategory::Mutation);
        Self { base }
    }
}

impl Default for TwoHeads {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for TwoHeads {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(TwoHeads::new().get_name(), "Two Heads"); }
    #[test]
    fn category_is_correct() { assert_eq!(TwoHeads::new().get_category(), SkillCategory::Mutation); }
}
