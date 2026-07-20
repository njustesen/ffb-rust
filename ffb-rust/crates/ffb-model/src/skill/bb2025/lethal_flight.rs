/// 1:1 translation of com.fumbbl.ffb.skill.bb2025::LethalFlight.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct LethalFlight {
    pub base: Skill,
}

impl LethalFlight {
    pub fn new() -> Self {
        let base = Skill::new("Lethal Flight", SkillCategory::Devious);
        Self { base }
    }
}

impl Default for LethalFlight {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for LethalFlight {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_lethal_flight() {
        assert_eq!(LethalFlight::new().get_name(), "Lethal Flight");
    }

    #[test]
    fn category_is_devious() {
        assert_eq!(LethalFlight::new().get_category(), SkillCategory::Devious);
    }

    #[test]
    fn has_affects_either_armour_or_injury_on_ttm_property() {
        assert!(crate::enums::SkillId::LethalFlight.properties().contains(&"affectsEitherArmourOrInjuryOnTtm"));
    }
}
