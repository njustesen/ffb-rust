use crate::modifiers::armor_modifier::ArmorModifier;

/// 1:1 translation of com.fumbbl.ffb.factory.ArmorModifiers (Java interface → Rust trait).
/// Each edition implements this to provide its modifier set.
pub trait ArmorModifiers: Send + Sync {
    fn get_name(&self) -> &str;
    fn values(&self) -> Vec<Box<dyn ArmorModifier>>;
    fn all_values(&self) -> Vec<Box<dyn ArmorModifier>>;
    fn set_use_all(&mut self, use_all: bool);
}
