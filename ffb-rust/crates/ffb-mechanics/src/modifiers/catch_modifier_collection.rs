use ffb_model::enums::Weather;
use crate::modifiers::catch_context::CatchContext;
use crate::modifiers::catch_modifier::CatchModifier;
use crate::modifiers::modifier_type::ModifierType;

/// 1:1 translation of com.fumbbl.ffb.modifiers.CatchModifierCollection (abstract base).
pub struct CatchModifierCollection {
    modifiers: Vec<CatchModifier>,
}

impl CatchModifierCollection {
    pub fn new() -> Self {
        let mut col = Self { modifiers: Vec::new() };
        col.init_base_modifiers();
        col
    }

    fn init_base_modifiers(&mut self) {
        for i in 1i32..=8 {
            let name = if i == 1 { "1 Tacklezone".to_string() } else { format!("{} Tacklezones", i) };
            self.add(CatchModifier::new(name, i, ModifierType::TACKLEZONE));
        }
        for i in 1i32..=11 {
            let name = if i == 1 { "1 Disturbing Presence".to_string() } else { format!("{} Disturbing Presences", i) };
            self.add(CatchModifier::new(name, i, ModifierType::DISTURBING_PRESENCE));
        }
        self.add(CatchModifier::new("Pouring Rain", 1, ModifierType::REGULAR)
            .with_predicate(|ctx| {
                ctx.game.field_model.weather == Weather::PouringRain
            }));
    }

    pub fn add(&mut self, modifier: CatchModifier) {
        self.modifiers.push(modifier);
    }

    pub fn get_modifiers(&self) -> &[CatchModifier] {
        &self.modifiers
    }

    pub fn get_modifiers_by_type(&self, modifier_type: ModifierType) -> Vec<&CatchModifier> {
        self.modifiers.iter().filter(|m| m.get_type() == modifier_type).collect()
    }

    pub fn find_applicable(&self, context: &CatchContext<'_>) -> Vec<&CatchModifier> {
        self.modifiers.iter().filter(|m| m.applies_to_context(context)).collect()
    }
}

impl Default for CatchModifierCollection {
    fn default() -> Self { Self::new() }
}
