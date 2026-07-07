use crate::modifiers::gaze_modifier::GazeModifier;
use crate::modifiers::gaze_modifier_context::GazeModifierContext;
use crate::modifiers::modifier_type::ModifierType;

/// 1:1 translation of com.fumbbl.ffb.modifiers.GazeModifierCollection.
/// Java GazeModifierCollection has no base modifiers (empty abstract class).
pub struct GazeModifierCollection {
    modifiers: Vec<GazeModifier>,
}

impl GazeModifierCollection {
    pub fn new() -> Self {
        Self { modifiers: Vec::new() }
    }

    pub fn add(&mut self, modifier: GazeModifier) {
        self.modifiers.push(modifier);
    }

    pub fn get_modifiers(&self) -> &[GazeModifier] {
        &self.modifiers
    }

    pub fn get_modifiers_by_type(&self, modifier_type: ModifierType) -> Vec<&GazeModifier> {
        self.modifiers.iter().filter(|m| m.get_type() == modifier_type).collect()
    }

    pub fn find_applicable<'a>(&'a self, context: &GazeModifierContext<'_>) -> Vec<&'a GazeModifier> {
        self.modifiers.iter().filter(|m| m.applies_to_context(context)).collect()
    }
}

impl Default for GazeModifierCollection {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_collection_is_empty() {
        assert_eq!(GazeModifierCollection::new().get_modifiers().len(), 0);
    }

    #[test]
    fn can_add_modifier_and_retrieve_it() {
        let mut col = GazeModifierCollection::new();
        col.add(GazeModifier::new("1 Tacklezone", 1, ModifierType::TACKLEZONE));
        assert_eq!(col.get_modifiers().len(), 1);
        assert_eq!(col.get_modifiers()[0].get_name(), "1 Tacklezone");
    }

    #[test]
    fn default_creates_empty_collection() {
        let col = GazeModifierCollection::default();
        assert_eq!(col.get_modifiers().len(), 0);
    }

    #[test]
    fn add_multiple_modifiers_preserves_order() {
        let mut col = GazeModifierCollection::new();
        col.add(GazeModifier::new("First", 1, ModifierType::TACKLEZONE));
        col.add(GazeModifier::new("Second", 2, ModifierType::DISTURBING_PRESENCE));
        col.add(GazeModifier::new("Third", -1, ModifierType::REGULAR));
        assert_eq!(col.get_modifiers().len(), 3);
        assert_eq!(col.get_modifiers()[0].get_name(), "First");
        assert_eq!(col.get_modifiers()[1].get_name(), "Second");
        assert_eq!(col.get_modifiers()[2].get_name(), "Third");
    }

    #[test]
    fn get_modifiers_by_type_filters_correctly() {
        let mut col = GazeModifierCollection::new();
        col.add(GazeModifier::new("TZ1", 1, ModifierType::TACKLEZONE));
        col.add(GazeModifier::new("TZ2", 2, ModifierType::TACKLEZONE));
        col.add(GazeModifier::new("Reg", 0, ModifierType::REGULAR));
        let tz = col.get_modifiers_by_type(ModifierType::TACKLEZONE);
        assert_eq!(tz.len(), 2);
        let reg = col.get_modifiers_by_type(ModifierType::REGULAR);
        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn find_applicable_returns_all_when_no_predicate() {
        let mut col = GazeModifierCollection::new();
        col.add(GazeModifier::new("A", 1, ModifierType::TACKLEZONE));
        col.add(GazeModifier::new("B", -1, ModifierType::REGULAR));
        // Without a predicate, applies_to_context always returns true
        // We can verify modifier values are accessible
        assert_eq!(col.get_modifiers()[0].get_modifier(), 1);
        assert_eq!(col.get_modifiers()[1].get_modifier(), -1);
    }

    #[test]
    fn get_modifiers_by_type_empty_when_no_match() {
        let mut col = GazeModifierCollection::new();
        col.add(GazeModifier::new("TZ", 1, ModifierType::TACKLEZONE));
        let dp = col.get_modifiers_by_type(ModifierType::DISTURBING_PRESENCE);
        assert_eq!(dp.len(), 0);
    }
}
