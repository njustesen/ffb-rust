use crate::modifiers::modifier_type::ModifierType;
use crate::modifiers::right_stuff_context::RightStuffContext;
use crate::modifiers::right_stuff_modifier::RightStuffModifier;

/// 1:1 translation of com.fumbbl.ffb.modifiers.RightStuffModifierCollection (abstract base).
/// Java RightStuffModifierCollection has no base modifiers.
pub struct RightStuffModifierCollection {
    modifiers: Vec<RightStuffModifier>,
}

impl RightStuffModifierCollection {
    pub fn new() -> Self {
        Self { modifiers: Vec::new() }
    }

    pub fn add(&mut self, modifier: RightStuffModifier) {
        self.modifiers.push(modifier);
    }

    pub fn get_modifiers(&self) -> &[RightStuffModifier] {
        &self.modifiers
    }

    pub fn get_modifiers_by_type(
        &self,
        modifier_type: ModifierType,
    ) -> Vec<&RightStuffModifier> {
        self.modifiers.iter().filter(|m| m.get_type() == modifier_type).collect()
    }

    pub fn find_applicable<'a>(
        &'a self,
        context: &RightStuffContext<'_>,
    ) -> Vec<&'a RightStuffModifier> {
        self.modifiers.iter().filter(|m| m.applies_to_context(context)).collect()
    }
}

impl Default for RightStuffModifierCollection {
    fn default() -> Self { Self::new() }
}
