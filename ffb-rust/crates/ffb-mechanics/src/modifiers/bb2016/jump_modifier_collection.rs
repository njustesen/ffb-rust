use crate::modifiers::jump_modifier::JumpModifier;
use crate::modifiers::jump_context::JumpContext;
use crate::modifiers::jump_modifier_collection::JumpModifierCollection as BaseJumpModifierCollection;

pub struct JumpModifierCollection {
    inner: BaseJumpModifierCollection,
}

impl JumpModifierCollection {
    pub fn new() -> Self { Self { inner: BaseJumpModifierCollection::new() } }
    pub fn get_modifiers(&self) -> &[JumpModifier] { self.inner.get_modifiers() }
    pub fn find_applicable<'a>(&'a self, ctx: &JumpContext<'_>) -> Vec<&'a JumpModifier> { self.inner.find_applicable(ctx) }
}

impl Default for JumpModifierCollection {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_zero_modifiers() {
        // bb2016 adds no extra jump modifiers on top of empty base
        assert_eq!(JumpModifierCollection::new().get_modifiers().len(), 0);
    }
}
