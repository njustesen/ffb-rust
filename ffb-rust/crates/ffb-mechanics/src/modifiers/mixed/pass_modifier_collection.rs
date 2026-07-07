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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_twenty_modifiers() {
        // base 1 very_sunny + 8 tacklezone + 11 disturbing_presence = 20 (no extra in mixed)
        assert_eq!(PassModifierCollection::new().get_modifiers().len(), 20);
    }

    #[test]
    fn includes_very_sunny_modifier() {
        assert!(PassModifierCollection::new().get_modifiers().iter().any(|m| m.get_name() == "Very Sunny"));
    }

    #[test]
    fn includes_single_tacklezone_modifier() {
        assert!(PassModifierCollection::new().get_modifiers().iter().any(|m| m.get_name() == "1 Tacklezone"));
    }
    #[test]
    fn includes_disturbing_presence_modifier() {
        let col = PassModifierCollection::new();
        assert!(col.get_modifiers().iter().any(|m| m.get_name().contains("Disturbing")));
    }

    #[test]
    fn all_modifiers_have_nonempty_names() {
        let col = PassModifierCollection::new();
        assert!(col.get_modifiers().iter().all(|m| !m.get_name().is_empty()));
    }
}
