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
