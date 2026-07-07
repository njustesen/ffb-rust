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
            inner.add(DodgeModifier::new_full(name, "1 for being marked with Prehensile Tail".to_string(), 1, i, ModifierType::PREHENSILE_TAIL, false));
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
        // base 8 tacklezone + 8 prehensile_tail = 16
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
        assert!(col.get_modifiers().iter().any(|m| m.get_name() == "3 Prehensile Tails"));
    }
    #[test]
    fn default_has_same_count_as_new() {
        assert_eq!(DodgeModifierCollection::default().get_modifiers().len(), DodgeModifierCollection::new().get_modifiers().len());
    }
}
