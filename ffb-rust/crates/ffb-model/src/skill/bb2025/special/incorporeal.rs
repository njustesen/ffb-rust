/// 1:1 translation of com.fumbbl.ffb.skill.bb2025.special::Incorporeal.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct Incorporeal {
    pub base: Skill,
}

impl Incorporeal {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Incorporeal", SkillCategory::Trait, SkillUsageType::OncePerGame);
        Self { base }
    }
}

impl Default for Incorporeal {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Incorporeal {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_incorporeal() {
        assert_eq!(Incorporeal::new().get_name(), "Incorporeal");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(Incorporeal::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn has_can_avoid_dodging_property() {
        assert!(crate::enums::SkillId::Incorporeal.properties().contains(&"canAvoidDodging"));
    }
}
