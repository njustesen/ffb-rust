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
}
