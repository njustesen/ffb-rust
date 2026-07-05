use crate::modifiers::right_stuff_modifier::RightStuffModifier;
use crate::modifiers::right_stuff_context::RightStuffContext;
use crate::modifiers::right_stuff_modifier_collection::RightStuffModifierCollection as BaseRightStuffModifierCollection;
use crate::modifiers::modifier_type::ModifierType;
use crate::pass_result::PassResult;

pub struct RightStuffModifierCollection {
    inner: BaseRightStuffModifierCollection,
}

impl RightStuffModifierCollection {
    pub fn new() -> Self {
        let mut inner = BaseRightStuffModifierCollection::new();
        inner.add(RightStuffModifier::new("Subpar Throw", 1, ModifierType::REGULAR)
            .with_predicate(|ctx| ctx.pass_result == Some(PassResult::INACCURATE)));
        inner.add(RightStuffModifier::new("Fumbled Throw", 1, ModifierType::REGULAR)
            .with_predicate(|ctx| ctx.pass_result == Some(PassResult::FUMBLE)));
        for i in 1i32..=8 {
            let name = if i == 1 { "1 Tacklezone".to_string() } else { format!("{} Tacklezones", i) };
            inner.add(RightStuffModifier::new_full(name, "1 for being marked".to_string(), i, ModifierType::TACKLEZONE));
        }
        Self { inner }
    }

    pub fn get_modifiers(&self) -> &[RightStuffModifier] { self.inner.get_modifiers() }
    pub fn find_applicable<'a>(&'a self, ctx: &RightStuffContext<'_>) -> Vec<&'a RightStuffModifier> { self.inner.find_applicable(ctx) }
}

impl Default for RightStuffModifierCollection {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_ten_modifiers() {
        // 2 pass result (subpar + fumbled) + 8 tacklezone = 10
        assert_eq!(RightStuffModifierCollection::new().get_modifiers().len(), 10);
    }

    #[test]
    fn includes_subpar_throw_modifier() {
        let col = RightStuffModifierCollection::new();
        assert!(col.get_modifiers().iter().any(|m| m.get_name() == "Subpar Throw"));
    }

    #[test]
    fn includes_fumbled_throw_modifier() {
        let col = RightStuffModifierCollection::new();
        assert!(col.get_modifiers().iter().any(|m| m.get_name() == "Fumbled Throw"));
    }
}
