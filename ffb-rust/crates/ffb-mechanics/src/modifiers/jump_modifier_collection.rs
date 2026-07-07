use crate::modifiers::jump_context::JumpContext;
use crate::modifiers::jump_modifier::JumpModifier;
use crate::modifiers::modifier_type::ModifierType;

/// 1:1 translation of com.fumbbl.ffb.modifiers.JumpModifierCollection (abstract base).
/// Java JumpModifierCollection has no base modifiers.
pub struct JumpModifierCollection {
    modifiers: Vec<JumpModifier>,
}

impl JumpModifierCollection {
    pub fn new() -> Self {
        Self { modifiers: Vec::new() }
    }

    pub fn add(&mut self, modifier: JumpModifier) {
        self.modifiers.push(modifier);
    }

    pub fn get_modifiers(&self) -> &[JumpModifier] {
        &self.modifiers
    }

    pub fn get_modifiers_by_type(&self, modifier_type: ModifierType) -> Vec<&JumpModifier> {
        self.modifiers.iter().filter(|m| m.get_type() == modifier_type).collect()
    }

    pub fn find_applicable<'a>(&'a self, context: &JumpContext<'_>) -> Vec<&'a JumpModifier> {
        self.modifiers.iter().filter(|m| m.applies_to_context(context)).collect()
    }
}

impl Default for JumpModifierCollection {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_collection_is_empty() {
        assert_eq!(JumpModifierCollection::new().get_modifiers().len(), 0);
    }

    #[test]
    fn can_add_and_retrieve_modifier() {
        let mut col = JumpModifierCollection::new();
        col.add(JumpModifier::new("1 Tacklezone", 1, ModifierType::TACKLEZONE));
        assert_eq!(col.get_modifiers().len(), 1);
        assert_eq!(col.get_modifiers()[0].get_name(), "1 Tacklezone");
    }

    #[test]
    fn default_creates_empty_collection() {
        let col = JumpModifierCollection::default();
        assert_eq!(col.get_modifiers().len(), 0);
    }

    #[test]
    fn add_multiple_modifiers_preserves_all() {
        let mut col = JumpModifierCollection::new();
        col.add(JumpModifier::new("TZ1", 1, ModifierType::TACKLEZONE));
        col.add(JumpModifier::new("Tail", 1, ModifierType::PREHENSILE_TAIL));
        col.add(JumpModifier::new("Reg", -1, ModifierType::REGULAR));
        assert_eq!(col.get_modifiers().len(), 3);
    }

    #[test]
    fn get_modifiers_by_type_filters_correctly() {
        let mut col = JumpModifierCollection::new();
        col.add(JumpModifier::new("TZ1", 1, ModifierType::TACKLEZONE));
        col.add(JumpModifier::new("TZ2", 2, ModifierType::TACKLEZONE));
        col.add(JumpModifier::new("Tail", 1, ModifierType::PREHENSILE_TAIL));
        let tz = col.get_modifiers_by_type(ModifierType::TACKLEZONE);
        assert_eq!(tz.len(), 2);
        let tail = col.get_modifiers_by_type(ModifierType::PREHENSILE_TAIL);
        assert_eq!(tail.len(), 1);
    }

    #[test]
    fn get_modifiers_by_type_empty_when_no_match() {
        let mut col = JumpModifierCollection::new();
        col.add(JumpModifier::new("TZ", 1, ModifierType::TACKLEZONE));
        assert_eq!(col.get_modifiers_by_type(ModifierType::REGULAR).len(), 0);
    }

    #[test]
    fn modifier_value_accessible() {
        let mut col = JumpModifierCollection::new();
        col.add(JumpModifier::new("TZ", -1, ModifierType::TACKLEZONE));
        assert_eq!(col.get_modifiers()[0].get_modifier(), -1);
    }
}
