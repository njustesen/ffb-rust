use crate::modifiers::catch_context::CatchContext;
use crate::modifiers::modifier_type::ModifierType;

/// 1:1 translation of com.fumbbl.ffb.modifiers.CatchModifier.
/// appliesToContext defaults to `true`; override by providing a custom predicate.
pub struct CatchModifier {
    pub name: String,
    pub reporting_string: String,
    pub modifier: i32,
    pub modifier_type: ModifierType,
    applies_to_context: Option<Box<dyn Fn(&CatchContext<'_>) -> bool + Send + Sync>>,
}

impl CatchModifier {
    pub fn new(name: impl Into<String>, modifier: i32, modifier_type: ModifierType) -> Self {
        let n = name.into();
        Self {
            reporting_string: n.clone(),
            name: n,
            modifier,
            modifier_type,
            applies_to_context: None,
        }
    }

    pub fn new_with_reporting(name: impl Into<String>, reporting_string: impl Into<String>, modifier: i32, modifier_type: ModifierType) -> Self {
        Self {
            name: name.into(),
            reporting_string: reporting_string.into(),
            modifier,
            modifier_type,
            applies_to_context: None,
        }
    }

    pub fn with_predicate(mut self, f: impl Fn(&CatchContext<'_>) -> bool + Send + Sync + 'static) -> Self {
        self.applies_to_context = Some(Box::new(f));
        self
    }

    pub fn get_modifier(&self) -> i32 { self.modifier }

    pub fn get_type(&self) -> ModifierType { self.modifier_type }

    pub fn get_name(&self) -> &str { &self.name }

    pub fn get_report_string(&self) -> &str { &self.reporting_string }

    pub fn is_modifier_included(&self) -> bool {
        self.modifier_type == ModifierType::DISTURBING_PRESENCE
            || self.modifier_type == ModifierType::TACKLEZONE
    }

    pub fn applies_to_context(&self, context: &CatchContext<'_>) -> bool {
        self.applies_to_context.as_ref().map(|f| f(context)).unwrap_or(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_name_and_modifier() {
        let m = CatchModifier::new("Rain", -1, ModifierType::REGULAR);
        assert_eq!(m.get_name(), "Rain");
        assert_eq!(m.get_modifier(), -1);
        assert_eq!(m.get_type(), ModifierType::REGULAR);
    }

    #[test]
    fn tacklezone_type_is_modifier_included() {
        let m = CatchModifier::new("1 Tacklezone", 1, ModifierType::TACKLEZONE);
        assert!(m.is_modifier_included());
    }

    #[test]
    fn regular_type_is_not_modifier_included() {
        let m = CatchModifier::new("Rain", -1, ModifierType::REGULAR);
        assert!(!m.is_modifier_included());
    }
}
