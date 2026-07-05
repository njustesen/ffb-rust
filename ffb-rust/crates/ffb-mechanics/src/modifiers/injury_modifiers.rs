use crate::modifiers::injury_modifier::InjuryModifier;

/// 1:1 translation of com.fumbbl.ffb.factory.InjuryModifiers (Java interface → Rust trait).
/// Each edition implements this to provide its modifier set.
pub trait InjuryModifiers: Send + Sync {
    fn get_name(&self) -> &str;
    fn values(&self) -> Vec<Box<dyn InjuryModifier>>;
    fn all_values(&self) -> Vec<Box<dyn InjuryModifier>>;
    fn set_use_all(&mut self, use_all: bool);
}
