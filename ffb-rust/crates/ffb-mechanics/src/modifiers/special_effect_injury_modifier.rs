use ffb_model::model::{Player, SpecialEffect};
use crate::modifiers::injury_modifier::InjuryModifier;
use crate::modifiers::injury_modifier_context::InjuryModifierContext;
use crate::modifiers::static_injury_modifier::StaticInjuryModifier;

/// 1:1 translation of com.fumbbl.ffb.modifiers.SpecialEffectInjuryModifier.
pub struct SpecialEffectInjuryModifier {
    inner: StaticInjuryModifier,
    pub effect: SpecialEffect,
}

impl SpecialEffectInjuryModifier {
    pub fn new(name: impl Into<String>, modifier: i32, niggling_injury_modifier: bool, effect: SpecialEffect) -> Self {
        Self { inner: StaticInjuryModifier::new(name, modifier, niggling_injury_modifier), effect }
    }

    pub fn get_effect(&self) -> SpecialEffect { self.effect }
}

impl InjuryModifier for SpecialEffectInjuryModifier {
    fn get_modifier(&self, attacker: Option<&Player>, defender: &Player) -> i32 { self.inner.get_modifier(attacker, defender) }
    fn get_name(&self) -> &str { self.inner.get_name() }
    fn is_niggling_injury_modifier(&self) -> bool { self.inner.is_niggling_injury_modifier() }
    fn applies_to_context(&self, context: &InjuryModifierContext<'_>) -> bool { self.inner.applies_to_context(context) }
    fn registered_to(&self) -> Option<&str> { self.inner.registered_to() }
    fn set_registered_to(&mut self, skill_id: Option<String>) { self.inner.set_registered_to(skill_id); }
    fn get_special_effect(&self) -> Option<SpecialEffect> { Some(self.effect) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_player() -> Player {
        use ffb_model::enums::{PlayerType, PlayerGender};
        Player {
            id: "p".into(), name: "p".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        }
    }

    #[test]
    fn stores_name_modifier_and_effect() {
        let m = SpecialEffectInjuryModifier::new("Lightning Stun", 1, false, SpecialEffect::LIGHTNING);
        let p = dummy_player();
        assert_eq!(m.get_name(), "Lightning Stun");
        assert_eq!(m.get_modifier(None, &p), 1);
        assert_eq!(m.get_effect(), SpecialEffect::LIGHTNING);
    }

    #[test]
    fn niggling_flag_propagates() {
        let m = SpecialEffectInjuryModifier::new("x", 0, true, SpecialEffect::BOMB);
        assert!(m.is_niggling_injury_modifier());
    }

    #[test]
    fn non_niggling_flag_is_false() {
        let m = SpecialEffectInjuryModifier::new("Fireball", 1, false, SpecialEffect::FIREBALL);
        assert!(!m.is_niggling_injury_modifier());
    }

    #[test]
    fn get_special_effect_returns_some() {
        let m = SpecialEffectInjuryModifier::new("Bomb", 2, false, SpecialEffect::BOMB);
        assert_eq!(m.get_special_effect(), Some(SpecialEffect::BOMB));
    }

    #[test]
    fn negative_modifier_stored_correctly() {
        let p = dummy_player();
        let m = SpecialEffectInjuryModifier::new("Heal", -1, false, SpecialEffect::LIGHTNING);
        assert_eq!(m.get_modifier(None, &p), -1);
    }

    #[test]
    fn registered_to_default_is_none() {
        let m = SpecialEffectInjuryModifier::new("X", 0, false, SpecialEffect::LIGHTNING);
        assert!(m.registered_to().is_none());
    }
}
