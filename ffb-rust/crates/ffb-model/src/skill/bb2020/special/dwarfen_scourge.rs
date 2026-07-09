/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::DwarfenScourge.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct DwarfenScourge {
    pub base: Skill,
}

impl DwarfenScourge {
    pub fn new() -> Self {
        let base = Skill::new("Dwarfen Scourge", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for DwarfenScourge {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for DwarfenScourge {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(DwarfenScourge::new().get_name(), "Dwarfen Scourge");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(DwarfenScourge::new().get_category(), SkillCategory::Trait);
    }
}
