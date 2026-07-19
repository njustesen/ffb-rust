/// 1:1 translation of com.fumbbl.ffb.skill.mixed::Grab.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Grab {
    pub base: Skill,
}

impl Grab {
    pub fn new() -> Self {
        let base = Skill::new("Grab", SkillCategory::Strength);
        Self { base }
    }

    /// Java `getSkillUseDescription()` override.
    pub fn get_skill_use_description(&self) -> Option<Vec<String>> {
        Some(vec!["Using Grab will allow to push the opponent into any open square, Side Step will be cancelled in any case".to_string()])
    }
}

impl Default for Grab {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Grab {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(Grab::new().get_name(), "Grab"); }
    #[test]
    fn category_is_correct() { assert_eq!(Grab::new().get_category(), SkillCategory::Strength); }
    #[test]
    fn skill_use_description_overridden() {
        assert_eq!(
            Grab::new().get_skill_use_description(),
            Some(vec!["Using Grab will allow to push the opponent into any open square, Side Step will be cancelled in any case".to_string()])
        );
    }
}
