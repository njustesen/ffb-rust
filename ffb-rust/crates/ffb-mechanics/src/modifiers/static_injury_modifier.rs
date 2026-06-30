use ffb_model::model::Player;
use crate::modifiers::injury_modifier::InjuryModifier;
use crate::modifiers::injury_modifier_context::InjuryModifierContext;

/// 1:1 translation of com.fumbbl.ffb.modifiers.StaticInjuryModifier.
pub struct StaticInjuryModifier {
    pub name: String,
    pub modifier: i32,
    pub niggling_injury_modifier: bool,
    pub registered_to: Option<String>,
    applies_to_context: Option<Box<dyn Fn(&InjuryModifierContext<'_>) -> bool + Send + Sync>>,
}

impl StaticInjuryModifier {
    pub fn new(name: impl Into<String>, modifier: i32, niggling_injury_modifier: bool) -> Self {
        Self { name: name.into(), modifier, niggling_injury_modifier, registered_to: None, applies_to_context: None }
    }

    pub fn with_predicate(mut self, f: impl Fn(&InjuryModifierContext<'_>) -> bool + Send + Sync + 'static) -> Self {
        self.applies_to_context = Some(Box::new(f));
        self
    }
}

impl InjuryModifier for StaticInjuryModifier {
    fn get_modifier(&self, _attacker: Option<&Player>, _defender: &Player) -> i32 { self.modifier }
    fn get_name(&self) -> &str { &self.name }
    fn is_niggling_injury_modifier(&self) -> bool { self.niggling_injury_modifier }
    fn applies_to_context(&self, context: &InjuryModifierContext<'_>) -> bool {
        self.applies_to_context.as_ref().map(|f| f(context)).unwrap_or(true)
    }
    fn registered_to(&self) -> Option<&str> { self.registered_to.as_deref() }
    fn set_registered_to(&mut self, skill_id: Option<String>) { self.registered_to = skill_id; }
}
