use ffb_model::model::Player;
use crate::modifiers::injury_modifier_context::InjuryModifierContext;

/// 1:1 translation of com.fumbbl.ffb.modifiers.InjuryModifier (Java interface → Rust trait).
pub trait InjuryModifier: Send + Sync {
    fn get_modifier(&self, attacker: Option<&Player>, defender: &Player) -> i32;
    fn get_name(&self) -> &str;
    fn is_niggling_injury_modifier(&self) -> bool;
    fn applies_to_context(&self, context: &InjuryModifierContext<'_>) -> bool;

    fn registered_to(&self) -> Option<&str> { None }
    fn set_registered_to(&mut self, _skill_id: Option<String>) {}
}
