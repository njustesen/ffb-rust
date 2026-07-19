/// 1:1 translation of com.fumbbl.ffb.skill.mixed::Dodge.
/// Deferred: Java's `postConstruct()` also calls `registerProperty`/`registerRerollSource`
/// (canRerollDodge, ignoreDefenderStumblesResult, DODGE reroll source) — left untranslated
/// because `Skill::register_reroll_source`/`register_property` are wired up nowhere else in
/// the ~300-file skill tree (0 callers in ffb-engine/ffb-mechanics), so this is part of a
/// systemic, cross-cutting postConstruct-registration gap rather than a Dodge-specific bug.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct Dodge {
    pub base: Skill,
}

impl Dodge {
    pub fn new() -> Self {
        let base = Skill::new("Dodge", SkillCategory::Agility);
        Self { base }
    }
}

impl Default for Dodge {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for Dodge {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn name_is_correct() { assert_eq!(Dodge::new().get_name(), "Dodge"); }
    #[test]
    fn category_is_correct() { assert_eq!(Dodge::new().get_category(), SkillCategory::Agility); }
}
