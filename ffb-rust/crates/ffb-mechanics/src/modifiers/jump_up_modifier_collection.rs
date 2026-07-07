use crate::modifiers::jump_up_context::JumpUpContext;
use crate::modifiers::jump_up_modifier::JumpUpModifier;
use crate::modifiers::modifier_type::ModifierType;

/// 1:1 translation of com.fumbbl.ffb.modifiers.JumpUpModifierCollection (abstract base).
/// Java JumpUpModifierCollection has no base modifiers.
pub struct JumpUpModifierCollection {
    modifiers: Vec<JumpUpModifier>,
}

impl JumpUpModifierCollection {
    pub fn new() -> Self {
        Self { modifiers: Vec::new() }
    }

    pub fn add(&mut self, modifier: JumpUpModifier) {
        self.modifiers.push(modifier);
    }

    pub fn get_modifiers(&self) -> &[JumpUpModifier] {
        &self.modifiers
    }

    pub fn get_modifiers_by_type(&self, modifier_type: ModifierType) -> Vec<&JumpUpModifier> {
        self.modifiers.iter().filter(|m| m.get_type() == modifier_type).collect()
    }

    pub fn find_applicable<'a>(
        &'a self,
        context: &JumpUpContext<'_>,
    ) -> Vec<&'a JumpUpModifier> {
        self.modifiers.iter().filter(|m| m.applies_to_context(context)).collect()
    }
}

impl Default for JumpUpModifierCollection {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_collection_is_empty() {
        assert_eq!(JumpUpModifierCollection::new().get_modifiers().len(), 0);
    }

    #[test]
    fn can_add_and_retrieve_modifier() {
        let mut col = JumpUpModifierCollection::new();
        col.add(JumpUpModifier::new("Jump Up", -2, ModifierType::REGULAR));
        assert_eq!(col.get_modifiers().len(), 1);
        assert_eq!(col.get_modifiers()[0].get_name(), "Jump Up");
    }

    #[test]
    fn default_creates_empty_collection() {
        let col = JumpUpModifierCollection::default();
        assert_eq!(col.get_modifiers().len(), 0);
    }

    #[test]
    fn add_multiple_modifiers_all_accessible() {
        let mut col = JumpUpModifierCollection::new();
        col.add(JumpUpModifier::new("Jump Up", -2, ModifierType::REGULAR));
        col.add(JumpUpModifier::new("TZ Penalty", 1, ModifierType::TACKLEZONE));
        assert_eq!(col.get_modifiers().len(), 2);
    }

    #[test]
    fn get_modifiers_by_type_filters_correctly() {
        let mut col = JumpUpModifierCollection::new();
        col.add(JumpUpModifier::new("Jump Up", -2, ModifierType::REGULAR));
        col.add(JumpUpModifier::new("TZ1", 1, ModifierType::TACKLEZONE));
        col.add(JumpUpModifier::new("TZ2", 2, ModifierType::TACKLEZONE));
        assert_eq!(col.get_modifiers_by_type(ModifierType::TACKLEZONE).len(), 2);
        assert_eq!(col.get_modifiers_by_type(ModifierType::REGULAR).len(), 1);
    }

    #[test]
    fn modifier_value_accessible() {
        let mut col = JumpUpModifierCollection::new();
        col.add(JumpUpModifier::new("Jump Up", -2, ModifierType::REGULAR));
        assert_eq!(col.get_modifiers()[0].get_modifier(), -2);
    }
}
