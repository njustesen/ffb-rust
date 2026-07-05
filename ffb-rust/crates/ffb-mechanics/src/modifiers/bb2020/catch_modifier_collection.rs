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
        inner.add(CatchModifier::new("Inaccurate Pass, Deviated Ball or Scatter", 1, ModifierType::REGULAR)
            .with_predicate(|ctx| matches!(ctx.catch_mode,
                CatchScatterThrowInMode::CatchBomb | CatchScatterThrowInMode::CatchScatter | CatchScatterThrowInMode::CatchKickoff)));
        inner.add(CatchModifier::new("Deflected Pass", 1, ModifierType::REGULAR)
            .with_predicate(|ctx| matches!(ctx.catch_mode,
                CatchScatterThrowInMode::Deflected | CatchScatterThrowInMode::DeflectedBomb)));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_twenty_three_modifiers() {
        // base 8 tacklezone + 11 disturbing_presence + 1 pouring_rain + 3 inaccurate/deflected/blast_it = 23
        assert_eq!(CatchModifierCollection::new().get_modifiers().len(), 23);
    }

    #[test]
    fn includes_deflected_pass_modifier() {
        let col = CatchModifierCollection::new();
        assert!(col.get_modifiers().iter().any(|m| m.get_name() == "Deflected Pass"));
    }

    #[test]
    fn includes_blast_it_modifier() {
        let col = CatchModifierCollection::new();
        assert!(col.get_modifiers().iter().any(|m| m.get_name() == "Blast It!"));
    }
}
