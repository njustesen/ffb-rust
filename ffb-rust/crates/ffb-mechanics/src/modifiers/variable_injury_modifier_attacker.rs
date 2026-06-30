use ffb_model::model::Player;
use crate::modifiers::injury_modifier::InjuryModifier;
use crate::modifiers::injury_modifier_context::InjuryModifierContext;

/// 1:1 translation of com.fumbbl.ffb.modifiers.VariableInjuryModifierAttacker.
/// appliesToContext: context.isAttackerMode() && UtilCards.hasSkill(attacker, registeredTo)
pub struct VariableInjuryModifierAttacker {
    pub name: String,
    pub niggling_injury_modifier: bool,
    pub registered_to: Option<String>,
}

impl VariableInjuryModifierAttacker {
    pub fn new(name: impl Into<String>, niggling_injury_modifier: bool) -> Self {
        Self { name: name.into(), niggling_injury_modifier, registered_to: None }
    }
}

impl InjuryModifier for VariableInjuryModifierAttacker {
    fn get_modifier(&self, attacker: Option<&Player>, _defender: &Player) -> i32 {
        attacker.map(|a| a.get_skill_int_value(self.registered_to.as_deref().unwrap_or(""))).unwrap_or(0)
    }
    fn get_name(&self) -> &str { &self.name }
    fn is_niggling_injury_modifier(&self) -> bool { self.niggling_injury_modifier }
    fn applies_to_context(&self, context: &InjuryModifierContext<'_>) -> bool {
        // TODO: full UtilCards.hasSkill check
        context.is_attacker_mode()
    }
    fn registered_to(&self) -> Option<&str> { self.registered_to.as_deref() }
    fn set_registered_to(&mut self, skill_id: Option<String>) { self.registered_to = skill_id; }
}
