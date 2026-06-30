use crate::modifiers::i_registration_aware_modifier::IRegistrationAwareModifier;

/// 1:1 translation of com.fumbbl.ffb.modifiers.RegistrationAwareModifier (Java abstract class).
pub struct RegistrationAwareModifier {
    pub registered_to: Option<String>,
}

impl RegistrationAwareModifier {
    pub fn new() -> Self {
        Self { registered_to: None }
    }
}

impl Default for RegistrationAwareModifier {
    fn default() -> Self { Self::new() }
}

impl IRegistrationAwareModifier for RegistrationAwareModifier {
    fn registered_to(&self) -> Option<&str> {
        self.registered_to.as_deref()
    }
    fn set_registered_to(&mut self, skill_id: Option<String>) {
        self.registered_to = skill_id;
    }
    fn is_registered_to_skill_with_property(&self, _property: &str) -> bool {
        // TODO: requires full Skill property lookup
        false
    }
}
