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
        inner.add(PassModifier::new("Blizzard", 0, ModifierType::REGULAR)
            .with_predicate(|ctx| ctx.game.field_model.weather == Weather::Blizzard));
        Self { inner }
    }

    pub fn get_modifiers(&self) -> &[PassModifier] { self.inner.get_modifiers() }
    pub fn find_applicable<'a>(&'a self, ctx: &PassContext<'_>) -> Vec<&'a PassModifier> { self.inner.find_applicable(ctx) }
}

impl Default for PassModifierCollection {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_twenty_one_modifiers() {
        // base 1 very_sunny + 8 tacklezone + 11 disturbing_presence + 1 blizzard = 21
        assert_eq!(PassModifierCollection::new().get_modifiers().len(), 21);
    }

    #[test]
    fn includes_blizzard_modifier() {
        let col = PassModifierCollection::new();
        assert!(col.get_modifiers().iter().any(|m| m.get_name() == "Blizzard"));
    }

    #[test]
    fn blizzard_is_regular_type() {
        use crate::modifiers::modifier_type::ModifierType;
        let col = PassModifierCollection::new();
        let blizzard = col.get_modifiers().iter().find(|m| m.get_name() == "Blizzard").unwrap();
        assert_eq!(blizzard.get_type(), ModifierType::REGULAR);
    }
}
