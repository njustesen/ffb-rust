/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::Swarming.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Swarming {
    pub base: Skill,
}

impl Swarming {
    pub fn new() -> Self {
        let base = Skill::new("Swarming", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for Swarming {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Swarming {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::SkillId;

    // bb2020/Swarming is SkillCategory.TRAIT (the bb2016 test's EXTRAORDINARY is bb2016-only).

    #[test]
    fn name_is_swarming() {
        assert_eq!(Swarming::new().get_name(), "Swarming");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(Swarming::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn has_can_sneak_extra_players_onto_pitch_property() {
        assert!(SkillId::Swarming.properties().contains(&"canSneakExtraPlayersOntoPitch"));
    }
}
