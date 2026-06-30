use crate::modifiers::pass_modifier::PassModifier;
use crate::modifiers::pass_context::PassContext;
use crate::modifiers::pass_modifier_collection::PassModifierCollection as BasePassModifierCollection;

pub struct PassModifierCollection {
    inner: BasePassModifierCollection,
}

impl PassModifierCollection {
    pub fn new() -> Self { Self { inner: BasePassModifierCollection::new() } }
    pub fn get_modifiers(&self) -> &[PassModifier] { self.inner.get_modifiers() }
    pub fn find_applicable<'a>(&'a self, ctx: &PassContext<'_>) -> Vec<&'a PassModifier> { self.inner.find_applicable(ctx) }
}

impl Default for PassModifierCollection {
    fn default() -> Self { Self::new() }
}
