use crate::modifiers::interception_modifier::InterceptionModifier;
use crate::modifiers::interception_context::InterceptionContext;
use crate::modifiers::interception_modifier_collection::InterceptionModifierCollection as BaseInterceptionModifierCollection;
use crate::modifiers::modifier_type::ModifierType;

pub struct InterceptionModifierCollection {
    inner: BaseInterceptionModifierCollection,
}

impl InterceptionModifierCollection {
    pub fn new() -> Self {
        let mut inner = BaseInterceptionModifierCollection::new();
        for i in 1i32..=8 {
            let name = if i == 1 { "1 Tacklezone".to_string() } else { format!("{} Tacklezones", i) };
            inner.add(InterceptionModifier::new(name, i, ModifierType::TACKLEZONE)
                .with_predicate(move |ctx| ctx.nr_of_tacklezones == i));
        }
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
    fn has_twenty_modifiers() {
        // base 11 disturbing_presence + 1 pouring_rain + 8 tacklezone = 20
        assert_eq!(InterceptionModifierCollection::new().get_modifiers().len(), 20);
    }

    #[test]
    fn includes_tacklezone_modifiers() {
        let col = InterceptionModifierCollection::new();
        assert!(col.get_modifiers().iter().any(|m| m.get_name() == "1 Tacklezone"));
        assert!(col.get_modifiers().iter().any(|m| m.get_name() == "8 Tacklezones"));
    }

    #[test]
    fn tacklezone_count_is_eight() {
        let col = InterceptionModifierCollection::new();
        use crate::modifiers::modifier_type::ModifierType;
        let tz_count = col.get_modifiers().iter().filter(|m| m.get_type() == ModifierType::TACKLEZONE).count();
        assert_eq!(tz_count, 8);
    }

    #[test]
    fn includes_eight_tacklezones_modifier() {
        let col = InterceptionModifierCollection::new();
        assert!(col.get_modifiers().iter().any(|m| m.get_name() == "8 Tacklezones"));
    }

    #[test]
    fn disturbing_presence_count_is_eleven() {
        let col = InterceptionModifierCollection::new();
        use crate::modifiers::modifier_type::ModifierType;
        let dp_count = col.get_modifiers().iter().filter(|m| m.get_type() == ModifierType::DISTURBING_PRESENCE).count();
        assert_eq!(dp_count, 11);
    }
}
