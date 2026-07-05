use ffb_model::model::SpecialEffect;
use crate::modifiers::injury_modifier::InjuryModifier;
use crate::modifiers::injury_modifiers::InjuryModifiers as InjuryModifiersTrait;
use crate::modifiers::special_effect_injury_modifier::SpecialEffectInjuryModifier;

/// 1:1 translation of com.fumbbl.ffb.factory.bb2020.InjuryModifiers.
/// BB2020: Fireball + Lightning (no nigglings); Bomb is legacy (only when use_all=true).
pub struct Bb2020InjuryModifiers {
    use_all: bool,
}

impl Bb2020InjuryModifiers {
    pub fn new() -> Self { Self { use_all: false } }
}

impl Default for Bb2020InjuryModifiers {
    fn default() -> Self { Self::new() }
}

impl InjuryModifiersTrait for Bb2020InjuryModifiers {
    fn get_name(&self) -> &str { "Bb2020InjuryModifiers" }

    fn values(&self) -> Vec<Box<dyn InjuryModifier>> {
        if self.use_all { self.all_values() } else { base_modifiers() }
    }

    fn all_values(&self) -> Vec<Box<dyn InjuryModifier>> {
        let mut v = legacy_modifiers();
        v.extend(base_modifiers());
        v
    }

    fn set_use_all(&mut self, use_all: bool) {
        self.use_all = use_all;
    }
}

fn legacy_modifiers() -> Vec<Box<dyn InjuryModifier>> {
    vec![
        Box::new(SpecialEffectInjuryModifier::new("Bomb", 1, false, SpecialEffect::BOMB)),
    ]
}

fn base_modifiers() -> Vec<Box<dyn InjuryModifier>> {
    vec![
        Box::new(SpecialEffectInjuryModifier::new("Fireball", 1, false, SpecialEffect::FIREBALL)),
        Box::new(SpecialEffectInjuryModifier::new("Lightning", 1, false, SpecialEffect::LIGHTNING)),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_excludes_bomb_by_default() {
        let m = Bb2020InjuryModifiers::new();
        let names: Vec<String> = m.values().into_iter().map(|m| m.get_name().to_string()).collect::<Vec<String>>();
        assert!(!names.iter().any(|n| n.as_str() == "Bomb"));
    }

    #[test]
    fn all_values_includes_bomb() {
        let m = Bb2020InjuryModifiers::new();
        let names: Vec<String> = m.all_values().into_iter().map(|m| m.get_name().to_string()).collect::<Vec<String>>();
        assert!(names.iter().any(|n| n.as_str() == "Bomb"));
    }

    #[test]
    fn set_use_all_includes_bomb_in_values() {
        let mut m = Bb2020InjuryModifiers::new();
        m.set_use_all(true);
        let names: Vec<String> = m.values().into_iter().map(|m| m.get_name().to_string()).collect::<Vec<String>>();
        assert!(names.iter().any(|n| n.as_str() == "Bomb"));
    }
}
