/// 1:1 translation of com.fumbbl.ffb.skill.mixed::Titchy.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Titchy {
    pub base: Skill,
}

impl Titchy {
    pub fn new() -> Self {
        let base = Skill::new("Titchy", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for Titchy {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Titchy {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mirrors ffb-java TitchySkillTest (written against the bb2016 class, where
    // the category is EXTRAORDINARY); category adapted to the mixed edition's
    // constructor (TRAIT). Property assertion verified against the mixed
    // edition's postConstruct, which registers hasNoTacklezoneForDodging (the
    // DodgeModifier it also registers has no live Rust equivalent). The Java
    // `is_bb2016_edition` test is skipped (edition annotations are covered
    // elsewhere).

    #[test]
    fn name_is_titchy() {
        assert_eq!(Titchy::new().get_name(), "Titchy");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(Titchy::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn has_no_tacklezone_for_dodging_property() {
        assert!(crate::enums::SkillId::Titchy.properties().contains(&"hasNoTacklezoneForDodging"));
    }
}
