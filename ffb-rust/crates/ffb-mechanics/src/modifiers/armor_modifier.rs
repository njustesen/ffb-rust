use ffb_model::model::{Player, SpecialEffect};
use crate::modifiers::armor_modifier_context::ArmorModifierContext;

/// 1:1 translation of com.fumbbl.ffb.modifiers.ArmorModifier (Java interface → Rust trait).
pub trait ArmorModifier: Send + Sync {
    fn get_modifier(&self, attacker: Option<&Player>, defender: &Player) -> i32;
    fn get_name(&self) -> &str;
    fn is_foul_assist_modifier(&self) -> bool;
    fn applies_to_context(&self, context: &ArmorModifierContext<'_>) -> bool;

    fn registered_to(&self) -> Option<&str> { None }
    fn set_registered_to(&mut self, _skill_id: Option<String>) {}

    /// Returns the SpecialEffect this modifier is tied to, if any.
    /// Overridden by SpecialEffectArmourModifier; all others return None.
    fn get_special_effect(&self) -> Option<SpecialEffect> { None }

    /// Java: isRegisteredToSkillWithProperty — checks if registered_to skill has property.
    /// headless: requires SkillFactory not yet ported
    fn is_registered_to_skill_with_property(&self, _property: &str) -> bool { false }
}
