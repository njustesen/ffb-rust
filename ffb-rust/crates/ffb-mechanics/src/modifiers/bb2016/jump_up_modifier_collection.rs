use crate::modifiers::jump_up_modifier::JumpUpModifier;
use crate::modifiers::jump_up_context::JumpUpContext;
use crate::modifiers::jump_up_modifier_collection::JumpUpModifierCollection as BaseJumpUpModifierCollection;
use crate::modifiers::modifier_type::ModifierType;

pub struct JumpUpModifierCollection {
    inner: BaseJumpUpModifierCollection,
}

impl JumpUpModifierCollection {
    pub fn new() -> Self {
        let mut inner = BaseJumpUpModifierCollection::new();
        inner.add(JumpUpModifier::new("Jump Up", -2, ModifierType::REGULAR));
        Self { inner }
    }

    pub fn get_modifiers(&self) -> &[JumpUpModifier] { self.inner.get_modifiers() }
    pub fn find_applicable<'a>(&'a self, ctx: &JumpUpContext<'_>) -> Vec<&'a JumpUpModifier> { self.inner.find_applicable(ctx) }
}

impl Default for JumpUpModifierCollection {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_one_modifier() {
        // bb2016 adds "Jump Up" (-2)
        assert_eq!(JumpUpModifierCollection::new().get_modifiers().len(), 1);
    }

    #[test]
    fn includes_jump_up_modifier() {
        let col = JumpUpModifierCollection::new();
        assert!(col.get_modifiers().iter().any(|m| m.get_name() == "Jump Up"));
    }

    #[test]
    fn jump_up_modifier_is_regular_type() {
        use crate::modifiers::modifier_type::ModifierType;
        let col = JumpUpModifierCollection::new();
        let ju = col.get_modifiers().iter().find(|m| m.get_name() == "Jump Up").unwrap();
        assert_eq!(ju.get_type(), ModifierType::REGULAR);
    }

    #[test]
    fn jump_up_modifier_value_is_minus_two() {
        let col = JumpUpModifierCollection::new();
        let ju = col.get_modifiers().iter().find(|m| m.get_name() == "Jump Up").unwrap();
        assert_eq!(ju.get_modifier(), -2);
    }

    #[test]
    fn regular_type_count_is_one() {
        use crate::modifiers::modifier_type::ModifierType;
        let col = JumpUpModifierCollection::new();
        let regular_count = col.get_modifiers().iter().filter(|m| m.get_type() == ModifierType::REGULAR).count();
        assert_eq!(regular_count, 1);
    }
}
