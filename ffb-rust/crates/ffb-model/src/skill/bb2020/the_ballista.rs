/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::TheBallista.
// NOTE: Java postConstruct calls registerRerollSource(...) twice (PASS and THROW_TEAM_MATE, both via
// ReRollSources.THE_BALLISTA). There is no live reroll-source lookup table in the Rust codebase to mirror this
// into (Skill::register_reroll_source is dead code), so this is left as a gap pending that infrastructure.
use crate::model::skill::skill::Skill;
use crate::enums::{SkillCategory, SkillUsageType};

pub struct TheBallista {
    pub base: Skill,
}

impl TheBallista {
    pub fn new() -> Self {
        let base = Skill::with_usage_type("The Ballista", SkillCategory::Trait, SkillUsageType::OncePerGame);
        Self { base }
    }
}

impl Default for TheBallista {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for TheBallista {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(TheBallista::new().get_name(), "The Ballista");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(TheBallista::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn usage_type_is_correct() {
        assert_eq!(TheBallista::new().get_skill_usage_type(), SkillUsageType::OncePerGame);
    }
}
