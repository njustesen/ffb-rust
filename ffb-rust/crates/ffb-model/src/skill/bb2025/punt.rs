/// 1:1 translation of com.fumbbl.ffb.skill.bb2025::Punt.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Punt {
    pub base: Skill,
}

impl Punt {
    pub fn new() -> Self {
        let base = Skill::new("Punt", SkillCategory::Passing);
        Self { base }
    }
}

impl Default for Punt {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Punt {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_punt() {
        assert_eq!(Punt::new().get_name(), "Punt");
    }

    #[test]
    fn category_is_passing() {
        assert_eq!(Punt::new().get_category(), SkillCategory::Passing);
    }

    #[test]
    fn has_can_punt_property() {
        assert!(crate::enums::SkillId::Punt.properties().contains(&"canPunt"));
    }
}
