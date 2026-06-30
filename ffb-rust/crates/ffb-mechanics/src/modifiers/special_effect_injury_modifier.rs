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
}
