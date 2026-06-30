use crate::modifiers::gaze_modifier::GazeModifier;
use crate::modifiers::gaze_modifier_context::GazeModifierContext;
use crate::modifiers::modifier_type::ModifierType;

pub struct GazeModifierCollection {
    modifiers: Vec<GazeModifier>,
}

impl GazeModifierCollection {
    pub fn new() -> Self {
        let mut col = Self { modifiers: Vec::new() };
        for i in 1i32..=8 {
            let name = if i == 1 { "1 Tacklezone".to_string() } else { format!("{} Tacklezones", i) };
            col.modifiers.push(GazeModifier::new_full(name, format!("{} for being marked", i), i - 1, i, ModifierType::TACKLEZONE));
        }
        col
    }

    pub fn get_modifiers(&self) -> &[GazeModifier] { &self.modifiers }
    pub fn find_applicable<'a>(&'a self, ctx: &GazeModifierContext<'_>) -> Vec<&'a GazeModifier> {
        self.modifiers.iter().filter(|m| m.applies_to_context(ctx)).collect()
    }
}

impl Default for GazeModifierCollection {
    fn default() -> Self { Self::new() }
}
