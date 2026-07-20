/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::Grab.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Grab {
    pub base: Skill,
}

impl Grab {
    pub fn new() -> Self {
        let base = Skill::new("Grab", SkillCategory::Strength);
        Self { base }
    }
}

impl Default for Grab {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Grab {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::SkillId;

    #[test]
    fn name_is_grab() {
        assert_eq!(Grab::new().get_name(), "Grab");
    }

    #[test]
    fn category_is_strength() {
        assert_eq!(Grab::new().get_category(), SkillCategory::Strength);
    }

    #[test]
    fn has_can_push_back_to_any_square_property() {
        assert!(SkillId::Grab.properties().contains(&"canPushBackToAnySquare"));
    }
}
