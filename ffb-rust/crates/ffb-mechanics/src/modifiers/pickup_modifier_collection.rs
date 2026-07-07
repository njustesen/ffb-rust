use ffb_model::enums::Weather;
use ffb_model::model::property::named_properties::NamedProperties;
use crate::modifiers::modifier_type::ModifierType;
use crate::modifiers::pickup_context::PickupContext;
use crate::modifiers::pickup_modifier::PickupModifier;

/// 1:1 translation of com.fumbbl.ffb.modifiers.PickupModifierCollection (abstract base).
pub struct PickupModifierCollection {
    modifiers: Vec<PickupModifier>,
}

impl PickupModifierCollection {
    pub fn new() -> Self {
        let mut col = Self { modifiers: Vec::new() };
        col.init_base_modifiers();
        col
    }

    fn init_base_modifiers(&mut self) {
        // Pouring Rain: applies unless player has ignoreWeatherWhenPickingUp skill property.
        self.add(
            PickupModifier::new("Pouring Rain", 1, ModifierType::REGULAR)
                .with_predicate(|ctx| {
                    ctx.game.field_model.weather == Weather::PouringRain
                        && !ctx.player.has_skill_property(NamedProperties::IGNORE_WEATHER_WHEN_PICKING_UP)
                }),
        );
        // 1-8 TACKLEZONE modifiers
        for i in 1i32..=8 {
            let name = if i == 1 {
                "1 Tacklezone".to_string()
            } else {
                format!("{} Tacklezones", i)
            };
            self.add(PickupModifier::new(name, i, ModifierType::TACKLEZONE));
        }
    }

    pub fn add(&mut self, modifier: PickupModifier) {
        self.modifiers.push(modifier);
    }

    pub fn get_modifiers(&self) -> &[PickupModifier] {
        &self.modifiers
    }

    pub fn get_modifiers_by_type(&self, modifier_type: ModifierType) -> Vec<&PickupModifier> {
        self.modifiers.iter().filter(|m| m.get_type() == modifier_type).collect()
    }

    pub fn find_applicable<'a>(&'a self, context: &PickupContext<'_>) -> Vec<&'a PickupModifier> {
        self.modifiers.iter().filter(|m| m.applies_to_context(context)).collect()
    }
}

impl Default for PickupModifierCollection {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_initializes_with_base_modifiers() {
        let col = PickupModifierCollection::new();
        assert!(!col.get_modifiers().is_empty());
    }

    #[test]
    fn get_modifiers_by_type_tacklezone_returns_8_entries() {
        let col = PickupModifierCollection::new();
        let tzones = col.get_modifiers_by_type(ModifierType::TACKLEZONE);
        assert_eq!(tzones.len(), 8);
    }

    #[test]
    fn add_increases_modifier_count() {
        let mut col = PickupModifierCollection::new();
        let count_before = col.get_modifiers().len();
        col.add(PickupModifier::new("extra", 1, ModifierType::REGULAR));
        assert_eq!(col.get_modifiers().len(), count_before + 1);
    }

    #[test]
    fn default_has_same_count_as_new() {
        assert_eq!(PickupModifierCollection::default().get_modifiers().len(), PickupModifierCollection::new().get_modifiers().len());
    }
}
