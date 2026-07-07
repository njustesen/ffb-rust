use ffb_model::model::property::named_properties::NamedProperties;
use crate::modifiers::interception_modifier::InterceptionModifier;
use crate::modifiers::interception_context::InterceptionContext;
use crate::modifiers::interception_modifier_collection::InterceptionModifierCollection as BaseInterceptionModifierCollection;
use crate::modifiers::modifier_type::ModifierType;
use crate::pass_result::PassResult;

pub struct InterceptionModifierCollection {
    inner: BaseInterceptionModifierCollection,
}

impl InterceptionModifierCollection {
    pub fn new() -> Self {
        let mut inner = BaseInterceptionModifierCollection::new();
        inner.add(InterceptionModifier::new("Accurate Pass", 3, ModifierType::REGULAR)
            .with_predicate(|ctx| ctx.pass_result == PassResult::ACCURATE));
        inner.add(InterceptionModifier::new("Inaccurate Pass", 2, ModifierType::REGULAR)
            .with_predicate(|ctx| ctx.pass_result == PassResult::INACCURATE));
        for i in 1i32..=8 {
            let name = if i == 1 { "1 Tacklezone".to_string() } else { format!("{} Tacklezones", i) };
            inner.add(InterceptionModifier::new(name, i, ModifierType::TACKLEZONE)
                .with_predicate(move |ctx| ctx.nr_of_tacklezones == i));
        }
        inner.add(InterceptionModifier::new("Thrower has Stunty", -1, ModifierType::REGULAR)
            .with_predicate(|ctx| {
                ctx.game.thrower().map(|t| t.has_skill_property(NamedProperties::PASSES_ARE_INTERCEPTED_EASIER)).unwrap_or(false)
                    && !ctx.bomb
            }));
        Self { inner }
    }

    pub fn get_modifiers(&self) -> &[InterceptionModifier] { self.inner.get_modifiers() }
    pub fn find_applicable<'a>(&'a self, ctx: &InterceptionContext<'_>) -> Vec<&'a InterceptionModifier> { self.inner.find_applicable(ctx) }
}

impl Default for InterceptionModifierCollection {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_twenty_three_modifiers() {
        // base 11 disturbing_presence + 1 pouring_rain + 2 pass_result + 8 tacklezone + 1 stunty = 23
        assert_eq!(InterceptionModifierCollection::new().get_modifiers().len(), 23);
    }

    #[test]
    fn includes_accurate_pass_modifier() {
        let col = InterceptionModifierCollection::new();
        assert!(col.get_modifiers().iter().any(|m| m.get_name() == "Accurate Pass"));
    }

    #[test]
    fn tacklezone_count_is_eight() {
        let col = InterceptionModifierCollection::new();
        let tz_count = col.get_modifiers().iter().filter(|m| m.get_type() == ModifierType::TACKLEZONE).count();
        assert_eq!(tz_count, 8);
    }

    #[test]
    fn includes_inaccurate_pass_modifier() {
        let col = InterceptionModifierCollection::new();
        assert!(col.get_modifiers().iter().any(|m| m.get_name() == "Inaccurate Pass"));
    }

    #[test]
    fn includes_stunty_modifier() {
        let col = InterceptionModifierCollection::new();
        assert!(col.get_modifiers().iter().any(|m| m.get_name() == "Thrower has Stunty"));
    }
}
