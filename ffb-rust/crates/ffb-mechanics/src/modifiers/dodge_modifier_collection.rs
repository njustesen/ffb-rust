use crate::modifiers::dodge_context::DodgeContext;
use crate::modifiers::dodge_modifier::DodgeModifier;
use crate::modifiers::modifier_type::ModifierType;

/// 1:1 translation of com.fumbbl.ffb.modifiers.DodgeModifierCollection (abstract base).
pub struct DodgeModifierCollection {
    modifiers: Vec<DodgeModifier>,
}

impl DodgeModifierCollection {
    pub fn new() -> Self {
        let mut col = Self { modifiers: Vec::new() };
        col.init_base_modifiers();
        col
    }

    fn init_base_modifiers(&mut self) {
        // Java base: 1-8 TACKLEZONE modifiers
        for i in 1i32..=8 {
            let name = if i == 1 {
                "1 Tacklezone".to_string()
            } else {
                format!("{} Tacklezones", i)
            };
            self.add(DodgeModifier::new(name, i, ModifierType::TACKLEZONE));
        }
    }

    pub fn add(&mut self, modifier: DodgeModifier) {
        self.modifiers.push(modifier);
    }

    pub fn get_modifiers(&self) -> &[DodgeModifier] {
        &self.modifiers
    }

    pub fn get_modifiers_by_type(&self, modifier_type: ModifierType) -> Vec<&DodgeModifier> {
        self.modifiers.iter().filter(|m| m.get_type() == modifier_type).collect()
    }

    pub fn find_applicable<'a>(&'a self, context: &DodgeContext<'_>) -> Vec<&'a DodgeModifier> {
        self.modifiers.iter().filter(|m| m.applies_to_context(context)).collect()
    }
}

impl Default for DodgeModifierCollection {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_eight_base_modifiers() {
        assert_eq!(DodgeModifierCollection::new().get_modifiers().len(), 8);
    }

    #[test]
    fn includes_single_tacklezone_modifier() {
        assert!(DodgeModifierCollection::new().get_modifiers().iter().any(|m| m.get_name() == "1 Tacklezone"));
    }

    #[test]
    fn all_base_modifiers_are_tacklezone_type() {
        let col = DodgeModifierCollection::new();
        assert!(col.get_modifiers().iter().all(|m| m.get_type() == ModifierType::TACKLEZONE));
    }
}
