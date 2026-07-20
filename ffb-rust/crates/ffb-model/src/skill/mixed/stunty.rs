/// 1:1 translation of com.fumbbl.ffb.skill.mixed::Stunty.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Stunty {
    pub base: Skill,
}

impl Stunty {
    pub fn new() -> Self {
        let base = Skill::new("Stunty", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for Stunty {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Stunty {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mirrors ffb-java StuntySkillTest (written against the bb2016 class, where
    // the category is EXTRAORDINARY); category adapted to the mixed edition's
    // constructor (TRAIT). Property assertions verified against the mixed
    // edition's postConstruct. The Java `is_bb2016_edition` test is skipped
    // (edition annotations are covered elsewhere).

    #[test]
    fn name_is_stunty() {
        assert_eq!(Stunty::new().get_name(), "Stunty");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(Stunty::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn has_ignore_tacklezones_when_dodging_property() {
        assert!(crate::enums::SkillId::Stunty.properties().contains(&"ignoreTacklezonesWhenDodging"));
    }

    #[test]
    fn has_is_hurt_more_easily_property() {
        assert!(crate::enums::SkillId::Stunty.properties().contains(&"isHurtMoreEasily"));
    }

    #[test]
    fn has_passes_are_intercepted_easier_property() {
        assert!(crate::enums::SkillId::Stunty.properties().contains(&"passesAreInterceptedEasier"));
    }
}
