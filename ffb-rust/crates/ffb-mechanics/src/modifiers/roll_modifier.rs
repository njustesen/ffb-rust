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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_name_and_modifier() {
        let m = RollModifier::new("Block", 1);
        assert_eq!(m.name, "Block");
        assert_eq!(m.get_modifier(), 1);
        assert!(!m.is_modifier_included());
    }

    #[test]
    fn with_report_sets_all_fields() {
        let m = RollModifier::with_report("Block", "Blocked!", 2, true);
        assert_eq!(m.get_report_string(), "Blocked!");
        assert_eq!(m.get_modifier(), 2);
        assert!(m.is_modifier_included());
    }

    #[test]
    fn get_multiplier_equals_modifier() {
        let m = RollModifier::new("x", 3);
        assert_eq!(m.get_multiplier(), 3);
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", RollModifier::default()).is_empty());
    }

}
