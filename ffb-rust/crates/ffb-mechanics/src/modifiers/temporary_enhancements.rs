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
