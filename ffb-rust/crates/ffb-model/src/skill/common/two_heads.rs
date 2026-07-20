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
    fn name_is_two_heads() {
        assert_eq!(TwoHeads::new().get_name(), "Two Heads");
    }

    #[test]
    fn category_is_mutation() {
        assert_eq!(TwoHeads::new().get_category(), SkillCategory::Mutation);
    }

    #[test]
    fn has_skill_properties_not_null() {
        // Java asserts getSkillProperties() is not null; the live Rust mechanism
        // always returns a slice (empty here — TwoHeads registers only a dodge modifier, no NamedProperties).
        assert!(crate::enums::SkillId::TwoHeads.properties().is_empty());
    }
}
