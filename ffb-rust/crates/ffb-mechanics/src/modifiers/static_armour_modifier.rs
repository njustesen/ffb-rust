use ffb_model::model::Player;
use crate::modifiers::armor_modifier::ArmorModifier;
use crate::modifiers::armor_modifier_context::ArmorModifierContext;

/// 1:1 translation of com.fumbbl.ffb.modifiers.StaticArmourModifier.
pub struct StaticArmourModifier {
    pub name: String,
    pub modifier: i32,
    pub foul_assist_modifier: bool,
    pub chainsaw: bool,
    pub registered_to: Option<String>,
    applies_to_context: Option<Box<dyn Fn(&ArmorModifierContext<'_>) -> bool + Send + Sync>>,
}

impl StaticArmourModifier {
    pub fn new(name: impl Into<String>, modifier: i32, foul_assist_modifier: bool) -> Self {
        Self { name: name.into(), modifier, foul_assist_modifier, chainsaw: false, registered_to: None, applies_to_context: None }
    }

    pub fn new_with_chainsaw(name: impl Into<String>, modifier: i32, foul_assist_modifier: bool, chainsaw: bool) -> Self {
        Self { name: name.into(), modifier, foul_assist_modifier, chainsaw, registered_to: None, applies_to_context: None }
    }

    pub fn with_predicate(mut self, f: impl Fn(&ArmorModifierContext<'_>) -> bool + Send + Sync + 'static) -> Self {
        self.applies_to_context = Some(Box::new(f));
        self
    }

    pub fn is_chainsaw(&self) -> bool { self.chainsaw }
}

impl ArmorModifier for StaticArmourModifier {
    fn get_modifier(&self, _attacker: Option<&Player>, _defender: &Player) -> i32 { self.modifier }
    fn get_name(&self) -> &str { &self.name }
    fn is_foul_assist_modifier(&self) -> bool { self.foul_assist_modifier }
    fn applies_to_context(&self, context: &ArmorModifierContext<'_>) -> bool {
        self.applies_to_context.as_ref().map(|f| f(context)).unwrap_or(true)
    }
    fn registered_to(&self) -> Option<&str> { self.registered_to.as_deref() }
    fn set_registered_to(&mut self, skill_id: Option<String>) { self.registered_to = skill_id; }
}
