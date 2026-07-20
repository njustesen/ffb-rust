/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::MonstrousMouth.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct MonstrousMouth {
    pub base: Skill,
}

impl MonstrousMouth {
    pub fn new() -> Self {
        let base = Skill::new("Monstrous Mouth", SkillCategory::Mutation);
        Self { base }
    }
}

impl Default for MonstrousMouth {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for MonstrousMouth {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::SkillId;

    // bb2020/MonstrousMouth is SkillCategory.MUTATION (the bb2016 test's EXTRAORDINARY is bb2016-only).

    #[test]
    fn name_is_monstrous_mouth() {
        assert_eq!(MonstrousMouth::new().get_name(), "Monstrous Mouth");
    }

    #[test]
    fn category_is_mutation() {
        assert_eq!(MonstrousMouth::new().get_category(), SkillCategory::Mutation);
    }

    #[test]
    fn skill_properties_are_not_null() {
        // Java: assertNotNull(skill.getSkillProperties()); the live Rust property table
        // is SkillId::MonstrousMouth.properties(), which always returns a valid slice.
        assert!(SkillId::MonstrousMouth.properties().iter().all(|p| !p.is_empty()));
    }
}
