/// 1:1 translation of com.fumbbl.ffb.modifiers.ModifierContext (Java interface → Rust marker trait).
/// All concrete context types (CatchContext, DodgeContext, etc.) implicitly implement this.
pub trait ModifierContext {}
