/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::WisdomOfTheWhiteDwarf.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct WisdomOfTheWhiteDwarf {
    pub base: Skill,
}

impl WisdomOfTheWhiteDwarf {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Wisdom of the White Dwarf", SkillCategory::Trait, SkillUsageType::OncePerTurnByTeamMate);
        Self { base }
    }
}

impl Default for WisdomOfTheWhiteDwarf {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for WisdomOfTheWhiteDwarf {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::SkillId;

    #[test]
    fn name_is_wisdom_of_the_white_dwarf() {
        assert_eq!(WisdomOfTheWhiteDwarf::new().get_name(), "Wisdom of the White Dwarf");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(WisdomOfTheWhiteDwarf::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn has_can_grant_skills_to_team_mates_property() {
        assert!(SkillId::WisdomOfTheWhiteDwarf.properties().contains(&"canGrantSkillsToTeamMates"));
    }

    #[test]
    fn usage_type_is_correct() {
        assert_eq!(WisdomOfTheWhiteDwarf::new().get_skill_usage_type(), SkillUsageType::OncePerTurnByTeamMate);
    }
}
