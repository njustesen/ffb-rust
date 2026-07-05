use ffb_model::model::CatchScatterThrowInMode;
use ffb_model::model::property::named_properties::NamedProperties;
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
        inner.add(CatchModifier::new("Accurate Pass", -1, ModifierType::REGULAR)
            .with_predicate(|ctx| {
                matches!(ctx.catch_mode,
                    CatchScatterThrowInMode::CatchAccurateBomb | CatchScatterThrowInMode::CatchAccuratePass)
                || (ctx.player.map(|p| p.has_skill_property(NamedProperties::ADD_BONUS_FOR_ACCURATE_PASS)).unwrap_or(false)
                    && matches!(ctx.catch_mode,
                        CatchScatterThrowInMode::CatchAccurateBombEmptySquare | CatchScatterThrowInMode::CatchAccuratePassEmptySquare))
            }));
        inner.add(CatchModifier::new("Hand Off", -1, ModifierType::REGULAR)
            .with_predicate(|ctx| ctx.catch_mode == CatchScatterThrowInMode::CatchHandOff));
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
        // base 8 tacklezone + 11 disturbing_presence + 1 pouring_rain + 2 accurate_pass/hand_off = 22
        assert_eq!(CatchModifierCollection::new().get_modifiers().len(), 22);
    }

    #[test]
    fn includes_accurate_pass_modifier() {
        let col = CatchModifierCollection::new();
        assert!(col.get_modifiers().iter().any(|m| m.get_name() == "Accurate Pass"));
    }

    #[test]
    fn includes_hand_off_modifier() {
        let col = CatchModifierCollection::new();
        assert!(col.get_modifiers().iter().any(|m| m.get_name() == "Hand Off"));
    }
}
