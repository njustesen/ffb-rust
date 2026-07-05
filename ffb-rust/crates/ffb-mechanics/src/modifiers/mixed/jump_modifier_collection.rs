use crate::modifiers::jump_modifier::JumpModifier;
use crate::modifiers::jump_context::JumpContext;
use crate::modifiers::jump_modifier_collection::JumpModifierCollection as BaseJumpModifierCollection;
use crate::modifiers::modifier_type::ModifierType;

pub struct JumpModifierCollection {
    inner: BaseJumpModifierCollection,
}

impl JumpModifierCollection {
    pub fn new() -> Self {
        let mut inner = BaseJumpModifierCollection::new();
        for i in 1i32..=8 {
            let name = if i == 1 { "1 Prehensile Tail".to_string() } else { format!("{} Prehensile Tails", i) };
            inner.add(JumpModifier::new_full(name, "1 for being marked with Prehensile Tail".to_string(), 1, i, ModifierType::PREHENSILE_TAIL));
        }
        for i in 1i32..=8 {
            let name = if i == 1 { "1 Tacklezone".to_string() } else { format!("{} Tacklezones", i) };
            inner.add(JumpModifier::new_full(name, "1 for being marked".to_string(), i, i, ModifierType::TACKLEZONE));
        }
        Self { inner }
    }

    pub fn get_modifiers(&self) -> &[JumpModifier] { self.inner.get_modifiers() }
    pub fn find_applicable<'a>(&'a self, ctx: &JumpContext<'_>) -> Vec<&'a JumpModifier> { self.inner.find_applicable(ctx) }
}

impl Default for JumpModifierCollection {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_sixteen_modifiers() {
        // 8 prehensile_tail + 8 tacklezone = 16
        assert_eq!(JumpModifierCollection::new().get_modifiers().len(), 16);
    }

    #[test]
    fn includes_prehensile_tail_modifier() {
        let col = JumpModifierCollection::new();
        assert!(col.get_modifiers().iter().any(|m| m.get_name() == "1 Prehensile Tail"));
    }

    #[test]
    fn includes_tacklezone_modifier() {
        let col = JumpModifierCollection::new();
        assert!(col.get_modifiers().iter().any(|m| m.get_name() == "1 Tacklezone"));
    }
}
