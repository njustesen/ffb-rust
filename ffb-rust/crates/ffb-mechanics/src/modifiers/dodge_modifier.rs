use crate::modifiers::dodge_context::DodgeContext;
use crate::modifiers::modifier_type::ModifierType;

/// 1:1 translation of com.fumbbl.ffb.modifiers.DodgeModifier.
pub struct DodgeModifier {
    pub name: String,
    pub reporting_string: String,
    pub modifier: i32,
    pub multiplier: i32,
    pub modifier_type: ModifierType,
    pub use_strength: bool,
    applies_to_context: Option<Box<dyn Fn(&DodgeContext<'_>) -> bool + Send + Sync>>,
}

impl DodgeModifier {
    pub fn new(name: impl Into<String>, modifier: i32, modifier_type: ModifierType) -> Self {
        let n = name.into();
        Self {
            reporting_string: n.clone(),
            name: n,
            modifier,
            multiplier: 1,
            modifier_type,
            use_strength: false,
            applies_to_context: None,
        }
    }

    pub fn new_with_use_strength(
        name: impl Into<String>,
        modifier: i32,
        modifier_type: ModifierType,
        use_strength: bool,
    ) -> Self {
        let n = name.into();
        Self {
            reporting_string: n.clone(),
            name: n,
            modifier,
            multiplier: 1,
            modifier_type,
            use_strength,
            applies_to_context: None,
        }
    }

    /// Java: DodgeModifier(String name, String reportingString, int modifier, int multiplier, ModifierType type, boolean useStrength)
    pub fn new_full(
        name: impl Into<String>,
        reporting_string: impl Into<String>,
        modifier: i32,
        multiplier: i32,
        modifier_type: ModifierType,
        use_strength: bool,
    ) -> Self {
        Self {
            name: name.into(),
            reporting_string: reporting_string.into(),
            modifier,
            multiplier,
            modifier_type,
            use_strength,
            applies_to_context: None,
        }
    }

    pub fn with_predicate(
        mut self,
        f: impl Fn(&DodgeContext<'_>) -> bool + Send + Sync + 'static,
    ) -> Self {
        self.applies_to_context = Some(Box::new(f));
        self
    }

    pub fn get_modifier(&self) -> i32 { self.modifier }
    pub fn get_multiplier(&self) -> i32 { self.multiplier }
    pub fn get_type(&self) -> ModifierType { self.modifier_type }
    pub fn get_name(&self) -> &str { &self.name }
    pub fn get_report_string(&self) -> &str { &self.reporting_string }

    pub fn is_modifier_included(&self) -> bool {
        self.modifier_type == ModifierType::TACKLEZONE
            || self.modifier_type == ModifierType::PREHENSILE_TAIL
    }

    pub fn is_use_strength(&self) -> bool { self.use_strength }

    pub fn applies_to_context(&self, context: &DodgeContext<'_>) -> bool {
        self.applies_to_context.as_ref().map(|f| f(context)).unwrap_or(true)
    }
}
