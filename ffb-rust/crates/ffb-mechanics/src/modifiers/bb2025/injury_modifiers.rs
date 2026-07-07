use ffb_model::model::SpecialEffect;
use crate::modifiers::injury_modifier::InjuryModifier;
use crate::modifiers::injury_modifiers::InjuryModifiers as InjuryModifiersTrait;
use crate::modifiers::special_effect_injury_modifier::SpecialEffectInjuryModifier;

/// 1:1 translation of com.fumbbl.ffb.factory.bb2025.InjuryModifiers.
/// BB2025: Fireball + Lightning only; no Bomb, no nigglings.
pub struct Bb2025InjuryModifiers;

impl InjuryModifiersTrait for Bb2025InjuryModifiers {
    fn get_name(&self) -> &str { "Bb2025InjuryModifiers" }

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
        Box::new(SpecialEffectInjuryModifier::new("Fireball", 1, false, SpecialEffect::FIREBALL)),
        Box::new(SpecialEffectInjuryModifier::new("Lightning", 1, false, SpecialEffect::LIGHTNING)),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_has_two_modifiers() {
        assert_eq!(Bb2025InjuryModifiers.values().len(), 2);
    }

    #[test]
    fn no_niggling_modifiers() {
        let count = Bb2025InjuryModifiers.values().iter()
            .filter(|m| m.is_niggling_injury_modifier()).count();
        assert_eq!(count, 0);
    }

    #[test]
    fn has_fireball_and_lightning() {
        let names: Vec<String> = Bb2025InjuryModifiers.values().into_iter().map(|m| m.get_name().to_string()).collect::<Vec<String>>();
        assert!(names.iter().any(|n| n.as_str() == "Fireball"));
        assert!(names.iter().any(|n| n.as_str() == "Lightning"));
    }

    #[test]
    fn get_name_is_nonempty() {
        assert!(!Bb2025InjuryModifiers.get_name().is_empty());
    }

    #[test]
    fn values_and_all_values_return_same_count() {
        assert_eq!(Bb2025InjuryModifiers.values().len(), Bb2025InjuryModifiers.all_values().len());
    }
}
