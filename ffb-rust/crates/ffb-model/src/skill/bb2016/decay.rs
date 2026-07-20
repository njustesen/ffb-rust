/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::Decay.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Decay {
    pub base: Skill,
}

impl Decay {
    pub fn new() -> Self {
        let base = Skill::new("Decay", SkillCategory::Extraordinary);
        Self { base }
    }
}

impl Default for Decay {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Decay {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::SkillId;

    #[test]
    fn name_is_decay() {
        assert_eq!(Decay::new().get_name(), "Decay");
    }

    #[test]
    fn category_is_extraordinary() {
        assert_eq!(Decay::new().get_category(), SkillCategory::Extraordinary);
    }

    #[test]
    fn has_requires_second_casualty_roll_property() {
        assert!(SkillId::Decay.properties().contains(&"requiresSecondCasualtyRoll"));
    }
}
