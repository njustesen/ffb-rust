/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::BalefulHex.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct BalefulHex {
    pub base: Skill,
}

impl BalefulHex {
    pub fn new() -> Self {
        let base = Skill::new("Baleful Hex", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for BalefulHex {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for BalefulHex {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(BalefulHex::new().get_name(), "Baleful Hex"); }
    #[test]
    fn category_is_correct() { assert_eq!(BalefulHex::new().get_category(), SkillCategory::Trait); }
}
