/// 1:1 translation of com.fumbbl.ffb.skill.mixed::Decay.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Decay {
    pub base: Skill,
}

impl Decay {
    pub fn new() -> Self {
        let base = Skill::new("Decay", SkillCategory::Trait);
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

    #[test]
    fn name_is_decay() {
        assert_eq!(Decay::new().get_name(), "Decay");
    }

    // Java test (bb2016 class) asserts EXTRAORDINARY; the mixed edition class mirrored
    // here uses SkillCategory.TRAIT.
    #[test]
    fn category_is_trait() {
        assert_eq!(Decay::new().get_category(), SkillCategory::Trait);
    }

    // Java test (bb2016 class) checks requiresSecondCasualtyRoll, which only the bb2016
    // edition registers. The mixed edition registers CancelSkillProperty(allowsRaisingLineman).
    #[test]
    fn has_cancels_allows_raising_lineman_property() {
        assert!(crate::enums::SkillId::Decay.properties().contains(&"cancelsAllowsRaisingLineman"));
    }
}
