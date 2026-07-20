/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::Regeneration.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Regeneration {
    pub base: Skill,
}

impl Regeneration {
    pub fn new() -> Self {
        let base = Skill::new("Regeneration", SkillCategory::Extraordinary);
        Self { base }
    }
}

impl Default for Regeneration {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Regeneration {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::SkillId;

    #[test]
    fn name_is_regeneration() {
        assert_eq!(Regeneration::new().get_name(), "Regeneration");
    }

    #[test]
    fn category_is_extraordinary() {
        assert_eq!(Regeneration::new().get_category(), SkillCategory::Extraordinary);
    }

    #[test]
    fn has_can_roll_to_save_from_injury_property() {
        assert!(SkillId::Regeneration.properties().contains(&"canRollToSaveFromInjury"));
    }
}
