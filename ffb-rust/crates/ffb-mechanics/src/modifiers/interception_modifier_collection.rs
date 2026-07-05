use ffb_model::enums::Weather;
use crate::modifiers::interception_context::InterceptionContext;
use crate::modifiers::interception_modifier::InterceptionModifier;
use crate::modifiers::modifier_type::ModifierType;

/// 1:1 translation of com.fumbbl.ffb.modifiers.InterceptionModifierCollection (abstract base).
pub struct InterceptionModifierCollection {
    modifiers: Vec<InterceptionModifier>,
}

impl InterceptionModifierCollection {
    pub fn new() -> Self {
        let mut col = Self { modifiers: Vec::new() };
        col.init_base_modifiers();
        col
    }

    fn init_base_modifiers(&mut self) {
        // Java base: 1-11 DISTURBING_PRESENCE modifiers
        for i in 1i32..=11 {
            let name = if i == 1 {
                "1 Disturbing Presence".to_string()
            } else {
                format!("{} Disturbing Presences", i)
            };
            self.add(InterceptionModifier::new(name, i, ModifierType::DISTURBING_PRESENCE));
        }
        // Pouring Rain weather modifier
        self.add(
            InterceptionModifier::new("Pouring Rain", 1, ModifierType::REGULAR)
                .with_predicate(|ctx| ctx.game.field_model.weather == Weather::PouringRain),
        );
    }

    pub fn add(&mut self, modifier: InterceptionModifier) {
        self.modifiers.push(modifier);
    }

    pub fn get_modifiers(&self) -> &[InterceptionModifier] {
        &self.modifiers
    }

    pub fn get_modifiers_by_type(
        &self,
        modifier_type: ModifierType,
    ) -> Vec<&InterceptionModifier> {
        self.modifiers.iter().filter(|m| m.get_type() == modifier_type).collect()
    }

    pub fn find_applicable<'a>(
        &'a self,
        context: &InterceptionContext<'_>,
    ) -> Vec<&'a InterceptionModifier> {
        self.modifiers.iter().filter(|m| m.applies_to_context(context)).collect()
    }
}

impl Default for InterceptionModifierCollection {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_twelve_base_modifiers() {
        // 11 disturbing_presence + 1 pouring_rain = 12
        assert_eq!(InterceptionModifierCollection::new().get_modifiers().len(), 12);
    }

    #[test]
    fn includes_pouring_rain_modifier() {
        assert!(InterceptionModifierCollection::new().get_modifiers().iter().any(|m| m.get_name() == "Pouring Rain"));
    }

    #[test]
    fn includes_disturbing_presence_modifier() {
        assert!(InterceptionModifierCollection::new().get_modifiers().iter().any(|m| m.get_name() == "1 Disturbing Presence"));
    }
}
