use ffb_model::enums::Weather;
use crate::modifiers::pass_modifier::PassModifier;
use crate::modifiers::pass_context::PassContext;
use crate::modifiers::pass_modifier_collection::PassModifierCollection as BasePassModifierCollection;
use crate::modifiers::modifier_type::ModifierType;

pub struct PassModifierCollection {
    inner: BasePassModifierCollection,
}

impl PassModifierCollection {
    pub fn new() -> Self {
        let mut inner = BasePassModifierCollection::new();
        inner.add(PassModifier::new("Blizzard", 1, ModifierType::REGULAR)
            .with_predicate(|ctx| ctx.game.field_model.weather == Weather::Blizzard));
        Self { inner }
    }

    pub fn get_modifiers(&self) -> &[PassModifier] { self.inner.get_modifiers() }
    pub fn find_applicable<'a>(&'a self, ctx: &PassContext<'_>) -> Vec<&'a PassModifier> { self.inner.find_applicable(ctx) }
}

impl Default for PassModifierCollection {
    fn default() -> Self { Self::new() }
}
