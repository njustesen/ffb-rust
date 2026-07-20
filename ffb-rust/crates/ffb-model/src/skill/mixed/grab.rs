/// 1:1 translation of com.fumbbl.ffb.skill.mixed::Grab.
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

    /// Java `getSkillUseDescription()` override.
    pub fn get_skill_use_description(&self) -> Option<Vec<String>> {
        Some(vec!["Using Grab will allow to push the opponent into any open square, Side Step will be cancelled in any case".to_string()])
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
        assert!(crate::enums::SkillId::Grab.properties().contains(&"canPushBackToAnySquare"));
    }

    // Mixed Grab also registers CancelSkillProperty(canChooseOwnPushedBackSquare).
    #[test]
    fn has_cancels_can_choose_own_pushed_back_square_property() {
        assert!(crate::enums::SkillId::Grab.properties().contains(&"cancelsCanChooseOwnPushedBackSquare"));
    }

    #[test]
    fn skill_use_description_overridden() {
        assert_eq!(
            Grab::new().get_skill_use_description(),
            Some(vec!["Using Grab will allow to push the opponent into any open square, Side Step will be cancelled in any case".to_string()])
        );
    }
}
