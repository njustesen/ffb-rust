/// 1:1 translation of com.fumbbl.ffb.skill.mixed::NervesOfSteel.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct NervesOfSteel {
    pub base: Skill,
}

impl NervesOfSteel {
    pub fn new() -> Self {
        let base = Skill::new("Nerves of Steel", SkillCategory::Passing);
        Self { base }
    }
}

impl Default for NervesOfSteel {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for NervesOfSteel {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mirrors ffb-java NervesOfSteelSkillTest (written against the bb2016 class);
    // property assertions verified against the mixed edition's postConstruct,
    // which registers ignoreTacklezonesWhenPassing + ignoreTacklezonesWhenCatching.
    // The Java `is_bb2016_edition` test is skipped (edition annotations are
    // covered elsewhere). The Pass/Interception/Catch modifiers it registers have
    // no live Rust equivalent to assert against.

    #[test]
    fn name_is_nerves_of_steel() {
        assert_eq!(NervesOfSteel::new().get_name(), "Nerves of Steel");
    }

    #[test]
    fn category_is_passing() {
        assert_eq!(NervesOfSteel::new().get_category(), SkillCategory::Passing);
    }

    #[test]
    fn has_ignore_tacklezones_when_passing_property() {
        assert!(crate::enums::SkillId::NervesOfSteel.properties().contains(&"ignoreTacklezonesWhenPassing"));
    }

    #[test]
    fn has_ignore_tacklezones_when_catching_property() {
        assert!(crate::enums::SkillId::NervesOfSteel.properties().contains(&"ignoreTacklezonesWhenCatching"));
    }
}
