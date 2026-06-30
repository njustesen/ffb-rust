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
