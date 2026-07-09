/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::GhostlyFlames.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct GhostlyFlames {
    pub base: Skill,
}

impl GhostlyFlames {
    pub fn new() -> Self {
        let base = Skill::new("Ghostly Flames", SkillCategory::Trait);
        Self { base }
    }
}

impl Default for GhostlyFlames {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for GhostlyFlames {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(GhostlyFlames::new().get_name(), "Ghostly Flames");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(GhostlyFlames::new().get_category(), SkillCategory::Trait);
    }
}
