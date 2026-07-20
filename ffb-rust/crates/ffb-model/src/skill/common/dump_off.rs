/// 1:1 translation of com.fumbbl.ffb.skill.common::DumpOff.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct DumpOff {
    pub base: Skill,
}

impl DumpOff {
    pub fn new() -> Self {
        let base = Skill::new("Dump-Off", SkillCategory::Passing);
        Self { base }
    }
}

impl Default for DumpOff {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for DumpOff {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_dump_off() {
        assert_eq!(DumpOff::new().get_name(), "Dump-Off");
    }

    #[test]
    fn category_is_passing() {
        assert_eq!(DumpOff::new().get_category(), SkillCategory::Passing);
    }

    #[test]
    fn skill_properties_are_not_null() {
        // Java asserts getSkillProperties() is not null; the live Rust mechanism
        // always returns a slice (empty here — DumpOff registers no NamedProperties).
        assert!(crate::enums::SkillId::DumpOff.properties().is_empty());
    }
}
