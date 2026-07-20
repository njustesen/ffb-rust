/// 1:1 translation of com.fumbbl.ffb.skill.mixed::ThrowTeamMate.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct ThrowTeamMate {
    pub base: Skill,
}

impl ThrowTeamMate {
    pub fn new() -> Self {
        let base = Skill::new("Throw Team-Mate", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for ThrowTeamMate {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for ThrowTeamMate {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mirrors ffb-java ThrowTeamMateSkillTest (written against the bb2016 class,
    // where the category is EXTRAORDINARY); category adapted to the mixed
    // edition's constructor (TRAIT). Property assertion verified against the
    // mixed edition's postConstruct, which registers canThrowTeamMates. The Java
    // `is_bb2016_edition` test is skipped (edition annotations are covered
    // elsewhere).

    #[test]
    fn name_is_throw_team_mate() {
        assert_eq!(ThrowTeamMate::new().get_name(), "Throw Team-Mate");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(ThrowTeamMate::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn has_can_throw_team_mates_property() {
        assert!(crate::enums::SkillId::ThrowTeamMate.properties().contains(&"canThrowTeamMates"));
    }
}
