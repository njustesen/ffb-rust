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
