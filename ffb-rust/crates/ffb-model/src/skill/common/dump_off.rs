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
    fn name_is_correct() { assert_eq!(DumpOff::new().get_name(), "Dump-Off"); }
    #[test]
    fn category_is_correct() { assert_eq!(DumpOff::new().get_category(), SkillCategory::Passing); }
}
