/// 1:1 translation of com.fumbbl.ffb.modifiers.IRegistrationAwareModifier (Java interface → Rust trait).
/// Implemented on armor/injury modifier types via their registered_to field.
pub trait IRegistrationAwareModifier {
    fn registered_to(&self) -> Option<&str>;
    fn set_registered_to(&mut self, skill_id: Option<String>);
    fn is_registered_to_skill_with_property(&self, _property: &str) -> bool {
        // TODO: requires full Skill property lookup
        false
    }
}
