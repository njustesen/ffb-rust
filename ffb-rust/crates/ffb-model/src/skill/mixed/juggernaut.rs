/// 1:1 translation of com.fumbbl.ffb.skill.mixed::Juggernaut.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Juggernaut {
    pub base: Skill,
}

impl Juggernaut {
    pub fn new() -> Self {
        let base = Skill::new("Juggernaut", SkillCategory::Strength);
        Self { base }
    }
}

impl Default for Juggernaut {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Juggernaut {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(Juggernaut::new().get_name(), "Juggernaut"); }
    #[test]
    fn category_is_correct() { assert_eq!(Juggernaut::new().get_category(), SkillCategory::Strength); }
}
