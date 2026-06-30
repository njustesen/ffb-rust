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
