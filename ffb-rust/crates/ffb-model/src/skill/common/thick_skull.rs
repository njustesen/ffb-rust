/// 1:1 translation of com.fumbbl.ffb.skill.common::ThickSkull.
// NOTE: Java's postConstruct() also does:
//   registerModifier(new StaticInjuryModifier("Thick Skull", 0, false) {
//       @Override public boolean appliesToContext(InjuryModifierContext context) { return false; }
//   });
// `InjuryModifier` is stubbed as `String` in the Rust `Skill` struct (no modifier
// subsystem ported yet), so this registration cannot be translated with the
// current infra. Note the Java modifier's appliesToContext always returns
// false, i.e. it never actually applies — so behaviorally this is a no-op in
// Java too. Deferred until the injury modifier subsystem is ported.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;
use crate::model::property::{NamedProperties, NamedProperty};

pub struct ThickSkull {
    pub base: Skill,
}

impl ThickSkull {
    pub fn new() -> Self {
        let mut base = Skill::new("Thick Skull", SkillCategory::Strength);
        // Java postConstruct(): registerProperty(NamedProperties.convertKOToStunOn8);
        base.register_property(Box::new(NamedProperty::new(NamedProperties::CONVERT_KO_TO_STUN_ON_8)));
        Self { base }
    }
}

impl Default for ThickSkull {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for ThickSkull {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_thick_skull() {
        assert_eq!(ThickSkull::new().get_name(), "Thick Skull");
    }

    #[test]
    fn category_is_strength() {
        assert_eq!(ThickSkull::new().get_category(), SkillCategory::Strength);
    }

    #[test]
    fn has_convert_ko_to_stun_on_8_property() {
        assert!(crate::enums::SkillId::ThickSkull.properties().contains(&"convertKOToStunOn8"));
    }
}
