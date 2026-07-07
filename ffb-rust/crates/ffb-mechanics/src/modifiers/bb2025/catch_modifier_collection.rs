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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_twenty_two_modifiers() {
        // base 8 tacklezone + 11 disturbing_presence + 1 pouring_rain + 2 (inaccurate/blast_it) = 22
        assert_eq!(CatchModifierCollection::new().get_modifiers().len(), 22);
    }

    #[test]
    fn includes_inaccurate_pass_or_scatter() {
        let col = CatchModifierCollection::new();
        assert!(col.get_modifiers().iter().any(|m| m.get_name() == "Inaccurate Pass or Scatter"));
    }

    #[test]
    fn includes_blast_it_modifier() {
        let col = CatchModifierCollection::new();
        assert!(col.get_modifiers().iter().any(|m| m.get_name() == "Blast It!"));
    }

    #[test]
    fn disturbing_presence_count_is_eleven() {
        let col = CatchModifierCollection::new();
        let dp_count = col.get_modifiers().iter().filter(|m| m.get_type() == ModifierType::DISTURBING_PRESENCE).count();
        assert_eq!(dp_count, 11);
    }

    #[test]
    fn tacklezone_count_is_eight() {
        let col = CatchModifierCollection::new();
        let tz_count = col.get_modifiers().iter().filter(|m| m.get_type() == ModifierType::TACKLEZONE).count();
        assert_eq!(tz_count, 8);
    }
}
