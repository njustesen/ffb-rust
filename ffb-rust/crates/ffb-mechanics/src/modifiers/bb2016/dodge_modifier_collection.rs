use crate::modifiers::dodge_modifier::DodgeModifier;
use crate::modifiers::dodge_context::DodgeContext;
use crate::modifiers::dodge_modifier_collection::DodgeModifierCollection as BaseDodgeModifierCollection;
use crate::modifiers::modifier_type::ModifierType;

pub struct DodgeModifierCollection {
    inner: BaseDodgeModifierCollection,
}

impl DodgeModifierCollection {
    pub fn new() -> Self {
        let mut inner = BaseDodgeModifierCollection::new();
        for i in 1i32..=8 {
            let name = if i == 1 { "1 Prehensile Tail".to_string() } else { format!("{} Prehensile Tails", i) };
            // Java: modifiers/bb2016/DodgeModifierCollection.java uses the 3-arg
            // DodgeModifier(name, modifier, type) ctor, which sets both `modifier` AND
            // `multiplier` to the same value — i.e. under BB2016 the dodge penalty scales
            // with the marker count (unlike BB2020/2025's flat -1-per-tail reporting text).
            inner.add(DodgeModifier::new_full(name.clone(), name, i, i, ModifierType::PREHENSILE_TAIL, false));
        }
        Self { inner }
    }

    pub fn get_modifiers(&self) -> &[DodgeModifier] { self.inner.get_modifiers() }
    pub fn find_applicable<'a>(&'a self, ctx: &DodgeContext<'_>) -> Vec<&'a DodgeModifier> { self.inner.find_applicable(ctx) }
}

impl Default for DodgeModifierCollection {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_sixteen_modifiers() {
        // base 8 tacklezone + 8 prehensile tail = 16
        assert_eq!(DodgeModifierCollection::new().get_modifiers().len(), 16);
    }

    #[test]
    fn includes_prehensile_tail_modifier() {
        let col = DodgeModifierCollection::new();
        assert!(col.get_modifiers().iter().any(|m| m.get_name() == "1 Prehensile Tail"));
    }

    #[test]
    fn plural_prehensile_tails_for_count_above_one() {
        let col = DodgeModifierCollection::new();
        assert!(col.get_modifiers().iter().any(|m| m.get_name() == "2 Prehensile Tails"));
    }
    #[test]
    fn includes_eight_tacklezone_modifier() {
        let col = DodgeModifierCollection::new();
        assert!(col.get_modifiers().iter().any(|m| m.get_name() == "8 Tacklezones"));
    }

    #[test]
    fn all_modifiers_have_nonempty_names() {
        let col = DodgeModifierCollection::new();
        assert!(col.get_modifiers().iter().all(|m| !m.get_name().is_empty()));
    }

    /// Regression test: Java's `modifiers/bb2016/DodgeModifierCollection.java` uses the 3-arg
    /// `DodgeModifier(name, modifier, type)` constructor, which sets `modifier == multiplier`.
    /// Previously the Rust translation hardcoded `modifier: 1` for every count, so a BB2016
    /// dodge against 3 Prehensile Tail markers only applied a -1 penalty instead of the correct
    /// -3 (scaling with marker count, unlike BB2020/2025 which stay flat at 1 per instance).
    #[test]
    fn prehensile_tail_modifier_scales_with_marker_count() {
        let col = DodgeModifierCollection::new();
        let three = col.get_modifiers().iter().find(|m| m.get_name() == "3 Prehensile Tails").unwrap();
        assert_eq!(three.get_modifier(), 3, "BB2016 Prehensile Tail penalty must scale with marker count");
        assert_eq!(three.get_multiplier(), 3);

        let one = col.get_modifiers().iter().find(|m| m.get_name() == "1 Prehensile Tail").unwrap();
        assert_eq!(one.get_modifier(), 1);
    }
}
