use ffb_model::enums::Weather;
use crate::modifiers::modifier_type::ModifierType;
use crate::modifiers::pass_context::PassContext;
use crate::modifiers::pass_modifier::PassModifier;

/// 1:1 translation of com.fumbbl.ffb.modifiers.PassModifierCollection (abstract base).
pub struct PassModifierCollection {
    modifiers: Vec<PassModifier>,
}

impl PassModifierCollection {
    pub fn new() -> Self {
        let mut col = Self { modifiers: Vec::new() };
        col.init_base_modifiers();
        col
    }

    fn init_base_modifiers(&mut self) {
        // Very Sunny weather modifier
        self.add(
            PassModifier::new("Very Sunny", 1, ModifierType::REGULAR)
                .with_predicate(|ctx| ctx.game.field_model.weather == Weather::VerySunny),
        );
        // 1-8 TACKLEZONE modifiers
        for i in 1i32..=8 {
            let name = if i == 1 {
                "1 Tacklezone".to_string()
            } else {
                format!("{} Tacklezones", i)
            };
            self.add(PassModifier::new(name, i, ModifierType::TACKLEZONE));
        }
        // 1-11 DISTURBING_PRESENCE modifiers
        for i in 1i32..=11 {
            let name = if i == 1 {
                "1 Disturbing Presence".to_string()
            } else {
                format!("{} Disturbing Presences", i)
            };
            self.add(PassModifier::new(name, i, ModifierType::DISTURBING_PRESENCE));
        }
    }

    pub fn add(&mut self, modifier: PassModifier) {
        self.modifiers.push(modifier);
    }

    pub fn get_modifiers(&self) -> &[PassModifier] {
        &self.modifiers
    }

    pub fn get_modifiers_by_type(&self, modifier_type: ModifierType) -> Vec<&PassModifier> {
        self.modifiers.iter().filter(|m| m.get_type() == modifier_type).collect()
    }

    pub fn find_applicable<'a>(&'a self, context: &PassContext<'_>) -> Vec<&'a PassModifier> {
        self.modifiers.iter().filter(|m| m.applies_to_context(context)).collect()
    }
}

impl Default for PassModifierCollection {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_initializes_with_base_modifiers() {
        let col = PassModifierCollection::new();
        assert!(!col.get_modifiers().is_empty());
    }

    #[test]
    fn get_modifiers_by_type_tacklezone_returns_8_entries() {
        let col = PassModifierCollection::new();
        let tzones = col.get_modifiers_by_type(ModifierType::TACKLEZONE);
        assert_eq!(tzones.len(), 8);
    }

    #[test]
    fn get_modifiers_by_type_disturbing_presence_returns_11_entries() {
        let col = PassModifierCollection::new();
        let dp = col.get_modifiers_by_type(ModifierType::DISTURBING_PRESENCE);
        assert_eq!(dp.len(), 11);
    }

    #[test]
    fn add_increases_modifier_count() {
        let mut col = PassModifierCollection::new();
        let count_before = col.get_modifiers().len();
        col.add(PassModifier::new("extra", 1, ModifierType::REGULAR));
        assert_eq!(col.get_modifiers().len(), count_before + 1);
    }

    #[test]
    fn default_has_same_count_as_new() {
        assert_eq!(PassModifierCollection::default().get_modifiers().len(), PassModifierCollection::new().get_modifiers().len());
    }
}
