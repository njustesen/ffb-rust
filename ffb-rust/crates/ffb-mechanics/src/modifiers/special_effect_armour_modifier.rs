use ffb_model::model::{Player, SpecialEffect};
use crate::modifiers::armor_modifier::ArmorModifier;
use crate::modifiers::armor_modifier_context::ArmorModifierContext;
use crate::modifiers::static_armour_modifier::StaticArmourModifier;

/// 1:1 translation of com.fumbbl.ffb.modifiers.SpecialEffectArmourModifier.
pub struct SpecialEffectArmourModifier {
    inner: StaticArmourModifier,
    pub effect: SpecialEffect,
}

impl SpecialEffectArmourModifier {
    pub fn new(name: impl Into<String>, modifier: i32, foul_assist_modifier: bool, effect: SpecialEffect) -> Self {
        Self { inner: StaticArmourModifier::new(name, modifier, foul_assist_modifier), effect }
    }

    pub fn get_effect(&self) -> SpecialEffect { self.effect }
}

impl ArmorModifier for SpecialEffectArmourModifier {
    fn get_modifier(&self, attacker: Option<&Player>, defender: &Player) -> i32 { self.inner.get_modifier(attacker, defender) }
    fn get_name(&self) -> &str { self.inner.get_name() }
    fn is_foul_assist_modifier(&self) -> bool { self.inner.is_foul_assist_modifier() }
    fn applies_to_context(&self, context: &ArmorModifierContext<'_>) -> bool { self.inner.applies_to_context(context) }
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
        let m = SpecialEffectArmourModifier::new("Lightning +3", 3, false, SpecialEffect::LIGHTNING);
        let p = dummy_player();
        assert_eq!(m.get_name(), "Lightning +3");
        assert_eq!(m.get_modifier(None, &p), 3);
        assert_eq!(m.get_effect(), SpecialEffect::LIGHTNING);
    }

    #[test]
    fn effect_field_stored_correctly() {
        let m = SpecialEffectArmourModifier::new("Fireball", 0, false, SpecialEffect::FIREBALL);
        assert_eq!(m.get_effect(), SpecialEffect::FIREBALL);
    }
}
