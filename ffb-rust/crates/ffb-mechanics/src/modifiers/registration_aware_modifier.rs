use ffb_model::enums::SkillId;
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
    fn is_registered_to_skill_with_property(&self, property: &str) -> bool {
        self.registered_to.as_deref()
            .and_then(|name| SkillId::from_class_name(name))
            .map(|id| id.properties().contains(&property))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_registered_to_skill_with_property_false_when_unregistered() {
        let m = RegistrationAwareModifier::new();
        assert!(!m.is_registered_to_skill_with_property("canLeap"));
    }

    #[test]
    fn is_registered_to_skill_with_property_true_for_leap_property() {
        let mut m = RegistrationAwareModifier::new();
        m.set_registered_to(Some("Leap".into()));
        assert!(m.is_registered_to_skill_with_property("canLeap"));
    }

    #[test]
    fn is_registered_to_skill_with_property_false_for_wrong_property() {
        let mut m = RegistrationAwareModifier::new();
        m.set_registered_to(Some("Leap".into()));
        assert!(!m.is_registered_to_skill_with_property("canAvoidFallingDown"));
    }

    #[test]
    fn set_registered_to_stores_value() {
        let mut m = RegistrationAwareModifier::new();
        m.set_registered_to(Some("DODGE".to_string()));
        assert_eq!(m.registered_to(), Some("DODGE"));
    }
}
