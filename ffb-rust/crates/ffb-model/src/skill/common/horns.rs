/// 1:1 translation of com.fumbbl.ffb.skill.common::Horns.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Horns {
    pub base: Skill,
}

impl Horns {
    pub fn new() -> Self {
        let base = Skill::new("Horns", SkillCategory::Mutation);
        Self { base }
    }
}

impl Default for Horns {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Horns {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_horns() {
        assert_eq!(Horns::new().get_name(), "Horns");
    }

    #[test]
    fn category_is_mutation() {
        assert_eq!(Horns::new().get_category(), SkillCategory::Mutation);
    }

    #[test]
    fn has_add_strength_on_blitz_property() {
        assert!(crate::enums::SkillId::Horns.properties().contains(&"addStrengthOnBlitz"));
    }
}
