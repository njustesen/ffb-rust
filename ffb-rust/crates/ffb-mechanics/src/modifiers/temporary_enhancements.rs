use std::collections::HashSet;
use crate::modifiers::temporary_stat_modifier::TemporaryStatModifier;

/// 1:1 translation of com.fumbbl.ffb.modifiers.TemporaryEnhancements.
/// SkillClassWithValue and ISkillProperty are not yet fully translated; using String placeholders.
pub struct TemporaryEnhancements {
    pub modifiers: HashSet<String>,
    pub skills: HashSet<String>,
    pub properties: HashSet<String>,
    pub stat_modifiers: Vec<TemporaryStatModifier>,
}

impl TemporaryEnhancements {
    pub fn new() -> Self {
        Self { modifiers: HashSet::new(), skills: HashSet::new(), properties: HashSet::new(), stat_modifiers: Vec::new() }
    }

    pub fn get_modifiers(&self) -> &HashSet<String> { &self.modifiers }
    pub fn get_skills(&self) -> &HashSet<String> { &self.skills }
    pub fn get_properties(&self) -> &HashSet<String> { &self.properties }
    pub fn get_stat_modifiers(&self) -> &[TemporaryStatModifier] { &self.stat_modifiers }
}

impl Default for TemporaryEnhancements {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_starts_empty() {
        let e = TemporaryEnhancements::new();
        assert!(e.get_modifiers().is_empty());
        assert!(e.get_skills().is_empty());
        assert!(e.get_properties().is_empty());
        assert!(e.get_stat_modifiers().is_empty());
    }

    #[test]
    fn can_add_modifier_string() {
        let mut e = TemporaryEnhancements::new();
        e.modifiers.insert("Block".into());
        assert!(e.get_modifiers().contains("Block"));
    }

    #[test]
    fn can_add_skill_and_property() {
        let mut e = TemporaryEnhancements::new();
        e.skills.insert("Dodge".into());
        e.properties.insert("canUseOwnTackleZone".into());
        assert!(e.get_skills().contains("Dodge"));
        assert!(e.get_properties().contains("canUseOwnTackleZone"));
    }
    #[test]
    fn default_is_same_as_new() {
        let e = TemporaryEnhancements::default();
        assert!(e.get_modifiers().is_empty());
        assert!(e.get_stat_modifiers().is_empty());
    }

    #[test]
    fn insert_skill_string_then_not_empty() {
        let mut te = TemporaryEnhancements::new();
        te.skills.insert("Dodge".to_string());
        assert!(!te.get_skills().is_empty());
    }
}
