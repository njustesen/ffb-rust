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
