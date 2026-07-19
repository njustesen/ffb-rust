/// 1:1 translation of com.fumbbl.ffb.skill.common::Sprint.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;
use crate::model::property::{NamedProperties, NamedProperty};

pub struct Sprint {
    pub base: Skill,
}

impl Sprint {
    pub fn new() -> Self {
        let mut base = Skill::new("Sprint", SkillCategory::Agility);
        // Java postConstruct(): registerProperty(NamedProperties.canMakeAnExtraGfi);
        base.register_property(Box::new(NamedProperty::new(NamedProperties::CAN_MAKE_AN_EXTRA_GFI)));
        Self { base }
    }
}

impl Default for Sprint {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Sprint {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(Sprint::new().get_name(), "Sprint"); }
    #[test]
    fn category_is_correct() { assert_eq!(Sprint::new().get_category(), SkillCategory::Agility); }
    #[test]
    fn registers_can_make_an_extra_gfi_property() {
        // Java Sprint.postConstruct() registers canMakeAnExtraGfi;
        // this would have failed before the fix since no property was registered.
        let s = Sprint::new();
        assert!(s.has_skill_property(NamedProperties::CAN_MAKE_AN_EXTRA_GFI));
    }
}
