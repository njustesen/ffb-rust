use ffb_model::model::SpecialEffect;
use crate::modifiers::injury_modifier::InjuryModifier;
use crate::modifiers::injury_modifiers::InjuryModifiers as InjuryModifiersTrait;
use crate::modifiers::special_effect_injury_modifier::SpecialEffectInjuryModifier;
use crate::modifiers::static_injury_modifier::StaticInjuryModifier;

/// 1:1 translation of com.fumbbl.ffb.factory.bb2016.InjuryModifiers.
/// BB2016: nigglings 1-5 + Fireball + Lightning.
pub struct Bb2016InjuryModifiers;

impl InjuryModifiersTrait for Bb2016InjuryModifiers {
    fn get_name(&self) -> &str { "Bb2016InjuryModifiers" }

    fn values(&self) -> Vec<Box<dyn InjuryModifier>> {
        all_modifiers()
    }

    fn all_values(&self) -> Vec<Box<dyn InjuryModifier>> {
        all_modifiers()
    }

    fn set_use_all(&mut self, _use_all: bool) {}
}

fn all_modifiers() -> Vec<Box<dyn InjuryModifier>> {
    vec![
        Box::new(StaticInjuryModifier::new("1 Niggling Injury", 1, true)),
        Box::new(StaticInjuryModifier::new("2 Niggling Injuries", 2, true)),
        Box::new(StaticInjuryModifier::new("3 Niggling Injuries", 3, true)),
        Box::new(StaticInjuryModifier::new("4 Niggling Injuries", 4, true)),
        Box::new(StaticInjuryModifier::new("5 Niggling Injuries", 5, true)),
        Box::new(SpecialEffectInjuryModifier::new("Fireball", 1, false, SpecialEffect::FIREBALL)),
        Box::new(SpecialEffectInjuryModifier::new("Lightning", 1, false, SpecialEffect::LIGHTNING)),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_has_seven_modifiers() {
        assert_eq!(Bb2016InjuryModifiers.values().len(), 7);
    }

    #[test]
    fn has_five_niggling_modifiers() {
        let count = Bb2016InjuryModifiers.values().iter()
            .filter(|m| m.is_niggling_injury_modifier()).count();
        assert_eq!(count, 5);
    }

    #[test]
    fn set_use_all_is_noop() {
        let mut m = Bb2016InjuryModifiers;
        let before = m.values().len();
        m.set_use_all(true);
        assert_eq!(m.values().len(), before);
    }
}
