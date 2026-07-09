/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::ToxinConnoisseur.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct ToxinConnoisseur {
    pub base: Skill,
}

impl ToxinConnoisseur {
    pub fn new() -> Self {
        let base = Skill::new("Toxin Connoisseur", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for ToxinConnoisseur {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for ToxinConnoisseur {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(ToxinConnoisseur::new().get_name(), "Toxin Connoisseur"); }
    #[test]
    fn category_is_correct() { assert_eq!(ToxinConnoisseur::new().get_category(), SkillCategory::Trait); }
}
