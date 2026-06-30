use crate::modifiers::modifier_type::ModifierType;
use crate::modifiers::right_stuff_context::RightStuffContext;

/// 1:1 translation of com.fumbbl.ffb.modifiers.RightStuffModifier.
pub struct RightStuffModifier {
    pub name: String,
    pub reporting_string: String,
    pub modifier: i32,
    pub modifier_type: ModifierType,
    applies_to_context: Option<Box<dyn Fn(&RightStuffContext<'_>) -> bool + Send + Sync>>,
}

impl RightStuffModifier {
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

    pub fn new_full(
        name: impl Into<String>,
        reporting_string: impl Into<String>,
        modifier: i32,
        modifier_type: ModifierType,
    ) -> Self {
        Self {
            name: name.into(),
            reporting_string: reporting_string.into(),
            modifier,
            modifier_type,
            applies_to_context: None,
        }
    }

    pub fn with_predicate(
        mut self,
        f: impl Fn(&RightStuffContext<'_>) -> bool + Send + Sync + 'static,
    ) -> Self {
        self.applies_to_context = Some(Box::new(f));
        self
    }

    pub fn get_modifier(&self) -> i32 { self.modifier }
    pub fn get_type(&self) -> ModifierType { self.modifier_type }
    pub fn get_name(&self) -> &str { &self.name }
    pub fn get_report_string(&self) -> &str { &self.reporting_string }

    pub fn is_modifier_included(&self) -> bool {
        self.modifier_type == ModifierType::TACKLEZONE
    }

    pub fn applies_to_context(&self, context: &RightStuffContext<'_>) -> bool {
        self.applies_to_context.as_ref().map(|f| f(context)).unwrap_or(true)
    }
}

impl Default for RightStuffModifier {
    fn default() -> Self {
        RightStuffModifier::new("", 0, ModifierType::REGULAR)
    }
}
