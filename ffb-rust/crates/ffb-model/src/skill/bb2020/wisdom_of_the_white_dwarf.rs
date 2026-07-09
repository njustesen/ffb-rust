/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::WisdomOfTheWhiteDwarf.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct WisdomOfTheWhiteDwarf {
    pub base: Skill,
}

impl WisdomOfTheWhiteDwarf {
    pub fn new() -> Self {
        let base = Skill::new("Wisdom of the White Dwarf", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for WisdomOfTheWhiteDwarf {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for WisdomOfTheWhiteDwarf {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(WisdomOfTheWhiteDwarf::new().get_name(), "Wisdom of the White Dwarf");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(WisdomOfTheWhiteDwarf::new().get_category(), SkillCategory::Trait);
    }
}
