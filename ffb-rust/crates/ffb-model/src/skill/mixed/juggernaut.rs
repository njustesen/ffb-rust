/// 1:1 translation of com.fumbbl.ffb.skill.mixed::Juggernaut.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Juggernaut {
    pub base: Skill,
}

impl Juggernaut {
    pub fn new() -> Self {
        let base = Skill::new("Juggernaut", SkillCategory::Strength);
        Self { base }
    }

    /// Java `getSkillUseDescription()` override.
    pub fn get_skill_use_description(&self) -> Option<Vec<String>> {
        Some(vec!["Using Juggernaut will convert the BOTH DOWN Block Result into a PUSHBACK.".to_string()])
    }
}

impl Default for Juggernaut {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Juggernaut {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_juggernaut() {
        assert_eq!(Juggernaut::new().get_name(), "Juggernaut");
    }

    #[test]
    fn category_is_strength() {
        assert_eq!(Juggernaut::new().get_category(), SkillCategory::Strength);
    }

    // Java test (bb2025 class) checks canConvertBothDownToPush, which the mixed edition
    // does not register; the mixed edition registers three CancelSkillProperties instead.
    #[test]
    fn has_cancels_can_take_down_players_with_him_on_both_down_property() {
        assert!(crate::enums::SkillId::Juggernaut.properties().contains(&"cancelsCanTakeDownPlayersWithHimOnBothDown"));
    }

    #[test]
    fn has_cancels_can_refuse_to_be_pushed_property() {
        assert!(crate::enums::SkillId::Juggernaut.properties().contains(&"cancelsCanRefuseToBePushed"));
    }

    #[test]
    fn has_cancels_prevent_opponent_following_up_property() {
        assert!(crate::enums::SkillId::Juggernaut.properties().contains(&"cancelsPreventOpponentFollowingUp"));
    }

    #[test]
    fn skill_use_description_overridden() {
        assert_eq!(
            Juggernaut::new().get_skill_use_description(),
            Some(vec!["Using Juggernaut will convert the BOTH DOWN Block Result into a PUSHBACK.".to_string()])
        );
    }
}
