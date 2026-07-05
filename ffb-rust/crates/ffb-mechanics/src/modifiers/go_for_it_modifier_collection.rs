use crate::modifiers::go_for_it_context::GoForItContext;
use crate::modifiers::go_for_it_modifier::GoForItModifier;

/// 1:1 translation of com.fumbbl.ffb.modifiers.GoForItModifierCollection (abstract base).
/// Java GoForItModifierCollection has no base modifiers.
pub struct GoForItModifierCollection {
    modifiers: Vec<GoForItModifier>,
}

impl GoForItModifierCollection {
    pub fn new() -> Self {
        Self { modifiers: Vec::new() }
    }

    pub fn add(&mut self, modifier: GoForItModifier) {
        self.modifiers.push(modifier);
    }

    pub fn get_modifiers(&self) -> &[GoForItModifier] {
        &self.modifiers
    }

    pub fn find_applicable<'a>(
        &'a self,
        context: &GoForItContext<'_>,
    ) -> Vec<&'a GoForItModifier> {
        self.modifiers.iter().filter(|m| m.applies_to_context(context)).collect()
    }
}

impl Default for GoForItModifierCollection {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_collection_is_empty() {
        assert_eq!(GoForItModifierCollection::new().get_modifiers().len(), 0);
    }

    #[test]
    fn can_add_and_retrieve_modifier() {
        let mut col = GoForItModifierCollection::new();
        col.add(GoForItModifier::new("Blizzard", -1));
        assert_eq!(col.get_modifiers().len(), 1);
        assert_eq!(col.get_modifiers()[0].get_name(), "Blizzard");
    }
}
