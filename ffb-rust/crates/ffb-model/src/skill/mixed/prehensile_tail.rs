/// 1:1 translation of com.fumbbl.ffb.skill.mixed::PrehensileTail.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct PrehensileTail {
    pub base: Skill,
}

impl PrehensileTail {
    pub fn new() -> Self {
        let base = Skill::new("Prehensile Tail", SkillCategory::Mutation);
        Self { base }
    }
}

impl Default for PrehensileTail {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for PrehensileTail {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mirrors ffb-java PrehensileTailSkillTest (written against the bb2016 class);
    // property assertions verified against the mixed edition's postConstruct,
    // which registers makesDodgingHarder + makesJumpingHarder.
    // The Java `is_bb2016_edition` test is skipped (edition annotations are
    // covered elsewhere).

    #[test]
    fn name_is_prehensile_tail() {
        assert_eq!(PrehensileTail::new().get_name(), "Prehensile Tail");
    }

    #[test]
    fn category_is_mutation() {
        assert_eq!(PrehensileTail::new().get_category(), SkillCategory::Mutation);
    }

    #[test]
    fn has_makes_dodging_harder_property() {
        assert!(crate::enums::SkillId::PrehensileTail.properties().contains(&"makesDodgingHarder"));
    }

    #[test]
    fn has_makes_jumping_harder_property() {
        assert!(crate::enums::SkillId::PrehensileTail.properties().contains(&"makesJumpingHarder"));
    }
}
