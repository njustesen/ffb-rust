use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.SpecialEffect.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpecialEffect {
    LIGHTNING,
    FIREBALL,
    ZAP,
    BOMB,
}

impl SpecialEffect {
    pub fn get_name(self) -> &'static str {
        match self {
            SpecialEffect::LIGHTNING => "lightning",
            SpecialEffect::FIREBALL => "fireball",
            SpecialEffect::ZAP => "zap",
            SpecialEffect::BOMB => "bomb",
        }
    }

    pub fn is_wizard_spell(self) -> bool {
        match self {
            SpecialEffect::LIGHTNING | SpecialEffect::FIREBALL | SpecialEffect::ZAP => true,
            SpecialEffect::BOMB => false,
        }
    }

    pub fn for_name(name: &str) -> Option<Self> {
        [Self::LIGHTNING, Self::FIREBALL, Self::ZAP, Self::BOMB]
            .iter().copied().find(|v| v.get_name().eq_ignore_ascii_case(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lightning_is_wizard_spell() {
        assert!(SpecialEffect::LIGHTNING.is_wizard_spell());
    }

    #[test]
    fn bomb_is_not_wizard_spell() {
        assert!(!SpecialEffect::BOMB.is_wizard_spell());
    }

    #[test]
    fn for_name_round_trip() {
        assert_eq!(SpecialEffect::for_name("lightning"), Some(SpecialEffect::LIGHTNING));
    }

    #[test]
    fn all_variants_get_name_round_trip() {
        let variants = [SpecialEffect::LIGHTNING, SpecialEffect::FIREBALL, SpecialEffect::ZAP, SpecialEffect::BOMB];
        for variant in variants {
            assert_eq!(SpecialEffect::for_name(variant.get_name()), Some(variant),
                "{:?} did not round-trip through get_name/for_name", variant);
        }
    }

    #[test]
    fn zap_and_fireball_are_wizard_spells() {
        assert!(SpecialEffect::ZAP.is_wizard_spell());
        assert!(SpecialEffect::FIREBALL.is_wizard_spell());
    }
}
