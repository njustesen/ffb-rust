/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::ThenIStartedBlastin.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct ThenIStartedBlastin {
    pub base: Skill,
}

impl ThenIStartedBlastin {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("\"Then I Started Blastin'!\"", SkillCategory::Trait, SkillUsageType::OncePerHalf);
        Self { base }
    }
}

impl Default for ThenIStartedBlastin {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for ThenIStartedBlastin {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::SkillId;

    #[test]
    fn name_contains_blastin() {
        assert!(ThenIStartedBlastin::new().get_name().contains("Blastin"));
    }

    #[test]
    fn category_is_trait() {
        assert_eq!(ThenIStartedBlastin::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn has_can_blast_remote_player_property() {
        assert!(SkillId::ThenIStartedBlastin.properties().contains(&"canBlastRemotePlayer"));
    }

    #[test]
    fn usage_type_is_correct() {
        assert_eq!(ThenIStartedBlastin::new().get_skill_usage_type(), SkillUsageType::OncePerHalf);
    }
}
