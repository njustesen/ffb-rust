/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::PilingOn.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct PilingOn {
    pub base: Skill,
}

impl PilingOn {
    pub fn new() -> Self {
        let base = Skill::new("Piling On", SkillCategory::Strength);
        Self { base }
    }

    /// Java `eligible()` — overridden to return false. Java comment: this should be removed
    /// but at the moment PilingOnBehavior is used to handle block knockdowns (e.g. for
    /// BothDown results), so this needs to be untangled first.
    pub fn eligible(&self) -> bool {
        false
    }
}

impl Default for PilingOn {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for PilingOn {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(PilingOn::new().get_name(), "Piling On");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(PilingOn::new().get_category(), SkillCategory::Strength);
    }

    #[test]
    fn eligible_is_false() {
        // Bug: Java PilingOn.eligible() overrides to return false, but the Rust struct had no
        // eligible() override at all, so it would fall back to the base Skill's `true` default.
        assert!(!PilingOn::new().eligible());
    }
}
