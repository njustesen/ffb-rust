use ffb_model::model::CatchScatterThrowInMode;
use crate::modifiers::catch_modifier::CatchModifier;
use crate::modifiers::catch_context::CatchContext;
use crate::modifiers::catch_modifier_collection::CatchModifierCollection as BaseCatchModifierCollection;
use crate::modifiers::modifier_type::ModifierType;

pub struct CatchModifierCollection {
    inner: BaseCatchModifierCollection,
}

impl CatchModifierCollection {
    pub fn new() -> Self {
        let mut inner = BaseCatchModifierCollection::new();
        inner.add(CatchModifier::new("Inaccurate Pass or Scatter", 1, ModifierType::REGULAR)
            .with_predicate(|ctx| matches!(ctx.catch_mode,
                CatchScatterThrowInMode::CatchBomb | CatchScatterThrowInMode::CatchScatter)));
        inner.add(CatchModifier::new("Blast It!", -1, ModifierType::REGULAR)
            .with_predicate(|ctx| {
                ctx.using_blast_it
                    && ctx.player.map(|p| ctx.game.active_team().has_player(&p.id)).unwrap_or(false)
                    && matches!(ctx.catch_mode,
                        CatchScatterThrowInMode::CatchScatter | CatchScatterThrowInMode::CatchMissedPass)
            }));
        Self { inner }
    }

    pub fn get_modifiers(&self) -> &[CatchModifier] { self.inner.get_modifiers() }
    pub fn find_applicable<'a>(&'a self, ctx: &CatchContext<'_>) -> Vec<&'a CatchModifier> { self.inner.find_applicable(ctx) }
}

impl Default for CatchModifierCollection {
    fn default() -> Self { Self::new() }
}
