use ffb_model::model::Player;
use crate::modifiers::injury_modifier::InjuryModifier;
use crate::modifiers::injury_modifier_context::InjuryModifierContext;

/// 1:1 translation of com.fumbbl.ffb.modifiers.VariableInjuryModifier (abstract).
/// getModifier uses relevantPlayer.getSkillIntValue(registeredTo).
/// This is the abstract base — use VariableInjuryModifierAttacker or Defender.
pub struct VariableInjuryModifier {
    pub name: String,
    pub niggling_injury_modifier: bool,
    pub registered_to: Option<String>,
    use_attacker: bool,
}

impl VariableInjuryModifier {
    pub fn new_attacker(name: impl Into<String>, niggling_injury_modifier: bool) -> Self {
        Self { name: name.into(), niggling_injury_modifier, registered_to: None, use_attacker: true }
    }

    pub fn new_defender(name: impl Into<String>, niggling_injury_modifier: bool) -> Self {
        Self { name: name.into(), niggling_injury_modifier, registered_to: None, use_attacker: false }
    }
}

impl InjuryModifier for VariableInjuryModifier {
    fn get_modifier(&self, attacker: Option<&Player>, defender: &Player) -> i32 {
        let prop = self.registered_to.as_deref().unwrap_or("");
        if self.use_attacker {
            attacker.map(|a| a.get_skill_int_value(prop)).unwrap_or(0)
        } else {
            defender.get_skill_int_value(prop)
        }
    }
    fn get_name(&self) -> &str { &self.name }
    fn is_niggling_injury_modifier(&self) -> bool { self.niggling_injury_modifier }
    fn applies_to_context(&self, _context: &InjuryModifierContext<'_>) -> bool { true }
    fn registered_to(&self) -> Option<&str> { self.registered_to.as_deref() }
    fn set_registered_to(&mut self, skill_id: Option<String>) { self.registered_to = skill_id; }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_attacker_stores_name_and_niggling_flag() {
        let m = VariableInjuryModifier::new_attacker("Mighty Blow", false);
        assert_eq!(m.get_name(), "Mighty Blow");
        assert!(!m.is_niggling_injury_modifier());
    }

    #[test]
    fn new_defender_niggling_flag_propagates() {
        let m = VariableInjuryModifier::new_defender("Niggling", true);
        assert!(m.is_niggling_injury_modifier());
    }

    #[test]
    fn registered_to_defaults_none() {
        let m = VariableInjuryModifier::new_attacker("x", false);
        assert!(m.registered_to().is_none());
    }

    #[test]
    fn set_registered_to_stores_value() {
        let mut m = VariableInjuryModifier::new_attacker("x", false);
        m.set_registered_to(Some("DODGE".to_string()));
        assert_eq!(m.registered_to(), Some("DODGE"));
    }
}
