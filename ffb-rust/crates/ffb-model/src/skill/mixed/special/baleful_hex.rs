/// 1:1 translation of com.fumbbl.ffb.skill.mixed.special::BalefulHex.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct BalefulHex {
    pub base: Skill,
}

impl BalefulHex {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("Baleful Hex", SkillCategory::Trait, SkillUsageType::OncePerGame);
        Self { base }
    }
}

impl Default for BalefulHex {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for BalefulHex {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_baleful_hex() {
        assert_eq!(BalefulHex::new().get_name(), "Baleful Hex");
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(BalefulHex::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn has_can_make_opponent_miss_turn_property() {
        assert!(crate::enums::SkillId::BalefulHex.properties().contains(&"canMakeOpponentMissTurn"));
    }

    #[test]
    fn class_name_is_baleful_hex() {
        assert_eq!(crate::enums::SkillId::BalefulHex.class_name(), "BalefulHex");
    }

    #[test]
    fn usage_type_is_once_per_game() {
        assert_eq!(BalefulHex::new().get_skill_usage_type(), SkillUsageType::OncePerGame);
    }
}
