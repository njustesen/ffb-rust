use ffb_model::enums::SkillId;

/// 1:1 translation of com.fumbbl.ffb.modifiers.IRegistrationAwareModifier (Java interface → Rust trait).
/// Implemented on armor/injury modifier types via their registered_to field.
pub trait IRegistrationAwareModifier {
    fn registered_to(&self) -> Option<&str>;
    fn set_registered_to(&mut self, skill_id: Option<String>);
    fn is_registered_to_skill_with_property(&self, property: &str) -> bool {
        self.registered_to()
            .and_then(|name| SkillId::from_class_name(name))
            .map(|id| id.properties().contains(&property))
            .unwrap_or(false)
    }
}
