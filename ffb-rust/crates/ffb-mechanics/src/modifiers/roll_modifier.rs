/// 1:1 translation of com.fumbbl.ffb.modifiers.RollModifier (base class behaviour only).
/// Concrete modifier types (PassModifier, DodgeModifier, etc.) are separate structs.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RollModifier {
    pub name: String,
    pub report_string: String,
    pub modifier: i32,
    pub is_modifier_included: bool,
}

impl RollModifier {
    pub fn new(name: impl Into<String>, modifier: i32) -> Self {
        let n = name.into();
        RollModifier {
            report_string: n.clone(),
            name: n,
            modifier,
            is_modifier_included: false,
        }
    }

    pub fn with_report(name: impl Into<String>, report_string: impl Into<String>, modifier: i32, is_modifier_included: bool) -> Self {
        RollModifier {
            name: name.into(),
            report_string: report_string.into(),
            modifier,
            is_modifier_included,
        }
    }

    pub fn get_modifier(&self) -> i32 { self.modifier }
    pub fn is_modifier_included(&self) -> bool { self.is_modifier_included }
    pub fn get_report_string(&self) -> &str { &self.report_string }
    pub fn get_multiplier(&self) -> i32 { self.modifier }
}

impl Default for RollModifier {
    fn default() -> Self {
        RollModifier::new("", 0)
    }
}
