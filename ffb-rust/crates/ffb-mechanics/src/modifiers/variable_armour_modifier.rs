use ffb_model::model::Player;
use crate::modifiers::armor_modifier::ArmorModifier;
use crate::modifiers::armor_modifier_context::ArmorModifierContext;

/// 1:1 translation of com.fumbbl.ffb.modifiers.VariableArmourModifier.
/// getModifier uses attacker.getSkillIntValue(registeredTo) — currently stubbed to 0.
pub struct VariableArmourModifier {
    pub name: String,
    pub foul_assist_modifier: bool,
    pub registered_to: Option<String>,
}

impl VariableArmourModifier {
    pub fn new(name: impl Into<String>, foul_assist_modifier: bool) -> Self {
        Self { name: name.into(), foul_assist_modifier, registered_to: None }
    }
}

impl ArmorModifier for VariableArmourModifier {
    fn get_modifier(&self, attacker: Option<&Player>, _defender: &Player) -> i32 {
        // TODO: attacker.getSkillIntValue(registeredTo) when SkillIntValue lookup is translated
        attacker.map(|a| a.get_skill_int_value(self.registered_to.as_deref().unwrap_or(""))).unwrap_or(0)
    }
    fn get_name(&self) -> &str { &self.name }
    fn is_foul_assist_modifier(&self) -> bool { self.foul_assist_modifier }
    fn applies_to_context(&self, _context: &ArmorModifierContext<'_>) -> bool { true }
    fn registered_to(&self) -> Option<&str> { self.registered_to.as_deref() }
    fn set_registered_to(&mut self, skill_id: Option<String>) { self.registered_to = skill_id; }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_name_and_foul_flag() {
        let m = VariableArmourModifier::new("Mighty Blow", false);
        assert_eq!(m.get_name(), "Mighty Blow");
        assert!(!m.is_foul_assist_modifier());
    }

    #[test]
    fn registered_to_defaults_none() {
        let m = VariableArmourModifier::new("x", false);
        assert!(m.registered_to().is_none());
    }

    #[test]
    fn foul_assist_flag_propagates() {
        let m_foul = VariableArmourModifier::new("Foul Assist", true);
        assert!(m_foul.is_foul_assist_modifier());
        let m_no_foul = VariableArmourModifier::new("x", false);
        assert!(!m_no_foul.is_foul_assist_modifier());
    }

    #[test]
    fn set_registered_to_stores_value() {
        let mut m = VariableArmourModifier::new("x", false);
        m.set_registered_to(Some("DODGE".to_string()));
        assert_eq!(m.registered_to(), Some("DODGE"));
    }

    #[test]
    fn set_registered_to_then_clear_to_none() {
        let mut m = VariableArmourModifier::new("x", false);
        m.set_registered_to(Some("DODGE".to_string()));
        m.set_registered_to(None);
        assert!(m.registered_to().is_none());
    }
}
