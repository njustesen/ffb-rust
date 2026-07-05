use crate::modifiers::jump_up_context::JumpUpContext;
use crate::modifiers::modifier_type::ModifierType;

/// 1:1 translation of com.fumbbl.ffb.modifiers.JumpUpModifier.
/// Note: Java JumpUpModifier has no reportingString field; getReportString() returns getName().
pub struct JumpUpModifier {
    pub name: String,
    pub modifier: i32,
    pub modifier_type: ModifierType,
    applies_to_context: Option<Box<dyn Fn(&JumpUpContext<'_>) -> bool + Send + Sync>>,
}

impl JumpUpModifier {
    pub fn new(name: impl Into<String>, modifier: i32, modifier_type: ModifierType) -> Self {
        Self {
            name: name.into(),
            modifier,
            modifier_type,
            applies_to_context: None,
        }
    }

    pub fn with_predicate(
        mut self,
        f: impl Fn(&JumpUpContext<'_>) -> bool + Send + Sync + 'static,
    ) -> Self {
        self.applies_to_context = Some(Box::new(f));
        self
    }

    pub fn get_modifier(&self) -> i32 { self.modifier }
    pub fn get_type(&self) -> ModifierType { self.modifier_type }
    pub fn get_name(&self) -> &str { &self.name }
    /// Java getReportString() returns getName()
    pub fn get_report_string(&self) -> &str { &self.name }

    /// Java isModifierIncluded() returns false
    pub fn is_modifier_included(&self) -> bool { false }

    pub fn applies_to_context(&self, context: &JumpUpContext<'_>) -> bool {
        self.applies_to_context.as_ref().map(|f| f(context)).unwrap_or(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_fields() {
        let m = JumpUpModifier::new("Jump Up", -2, ModifierType::REGULAR);
        assert_eq!(m.get_name(), "Jump Up");
        assert_eq!(m.get_modifier(), -2);
        assert_eq!(m.get_type(), ModifierType::REGULAR);
    }

    #[test]
    fn is_modifier_included_always_false() {
        assert!(!JumpUpModifier::new("x", 0, ModifierType::REGULAR).is_modifier_included());
        assert!(!JumpUpModifier::new("y", 1, ModifierType::TACKLEZONE).is_modifier_included());
    }

    #[test]
    fn report_string_equals_name() {
        let m = JumpUpModifier::new("Jump Up", 0, ModifierType::REGULAR);
        assert_eq!(m.get_report_string(), m.get_name());
    }
}
