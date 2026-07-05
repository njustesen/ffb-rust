use crate::modifiers::go_for_it_context::GoForItContext;
use crate::modifiers::modifier_type::ModifierType;

/// 1:1 translation of com.fumbbl.ffb.modifiers.GoForItModifier.
/// Java GoForItModifier type is always REGULAR; isModifierIncluded() returns false.
pub struct GoForItModifier {
    pub name: String,
    pub modifier: i32,
    applies_to_context: Option<Box<dyn Fn(&GoForItContext<'_>) -> bool + Send + Sync>>,
}

impl GoForItModifier {
    pub fn new(name: impl Into<String>, modifier: i32) -> Self {
        Self {
            name: name.into(),
            modifier,
            applies_to_context: None,
        }
    }

    pub fn with_predicate(
        mut self,
        f: impl Fn(&GoForItContext<'_>) -> bool + Send + Sync + 'static,
    ) -> Self {
        self.applies_to_context = Some(Box::new(f));
        self
    }

    pub fn get_modifier(&self) -> i32 { self.modifier }
    pub fn get_type(&self) -> ModifierType { ModifierType::REGULAR }
    pub fn get_name(&self) -> &str { &self.name }
    pub fn get_report_string(&self) -> &str { &self.name }

    /// Java isModifierIncluded() returns false
    pub fn is_modifier_included(&self) -> bool { false }

    pub fn applies_to_context(&self, context: &GoForItContext<'_>) -> bool {
        self.applies_to_context.as_ref().map(|f| f(context)).unwrap_or(true)
    }
}

impl Default for GoForItModifier {
    fn default() -> Self {
        GoForItModifier::new("", 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_name_and_modifier() {
        let m = GoForItModifier::new("Blizzard", -1);
        assert_eq!(m.get_name(), "Blizzard");
        assert_eq!(m.get_modifier(), -1);
    }

    #[test]
    fn type_is_always_regular() {
        assert_eq!(GoForItModifier::new("x", 0).get_type(), ModifierType::REGULAR);
    }

    #[test]
    fn is_modifier_included_always_false() {
        assert!(!GoForItModifier::new("x", 0).is_modifier_included());
    }
}
