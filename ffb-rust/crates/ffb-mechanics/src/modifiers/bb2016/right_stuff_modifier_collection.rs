use crate::modifiers::right_stuff_modifier::RightStuffModifier;
use crate::modifiers::right_stuff_context::RightStuffContext;
use crate::modifiers::right_stuff_modifier_collection::RightStuffModifierCollection as BaseRightStuffModifierCollection;
use crate::modifiers::modifier_type::ModifierType;

pub struct RightStuffModifierCollection {
    inner: BaseRightStuffModifierCollection,
}

impl RightStuffModifierCollection {
    pub fn new() -> Self {
        let mut inner = BaseRightStuffModifierCollection::new();
        inner.add(RightStuffModifier::new("Medium Kick", 1, ModifierType::REGULAR)
            .with_predicate(|ctx| ctx.ktm_range.as_deref() == Some("medium")));
        inner.add(RightStuffModifier::new("Long Kick", 2, ModifierType::REGULAR)
            .with_predicate(|ctx| ctx.ktm_range.as_deref() == Some("long")));
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
        // 2 kick range (medium + long) + 8 tacklezone = 10
        assert_eq!(RightStuffModifierCollection::new().get_modifiers().len(), 10);
    }

    #[test]
    fn includes_medium_kick_modifier() {
        let col = RightStuffModifierCollection::new();
        assert!(col.get_modifiers().iter().any(|m| m.get_name() == "Medium Kick"));
    }

    #[test]
    fn tacklezone_count_is_eight() {
        let col = RightStuffModifierCollection::new();
        let tz_count = col.get_modifiers().iter().filter(|m| m.get_type() == ModifierType::TACKLEZONE).count();
        assert_eq!(tz_count, 8);
    }

    #[test]
    fn includes_long_kick_modifier() {
        let col = RightStuffModifierCollection::new();
        assert!(col.get_modifiers().iter().any(|m| m.get_name() == "Long Kick"));
    }

    #[test]
    fn long_kick_is_regular_type() {
        let col = RightStuffModifierCollection::new();
        let lk = col.get_modifiers().iter().find(|m| m.get_name() == "Long Kick").unwrap();
        assert_eq!(lk.get_type(), ModifierType::REGULAR);
    }
}
