/// 1:1 translation of com.fumbbl.ffb.modifiers.StatBasedRollModifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatBasedRollModifier {
    pub name: String,
    pub value: i32,
}

impl StatBasedRollModifier {
    pub fn new(name: impl Into<String>, value: i32) -> Self {
        StatBasedRollModifier { name: name.into(), value }
    }

    pub fn get_modifier(&self) -> i32 { self.value }
    pub fn is_modifier_included(&self) -> bool { false }
    pub fn get_report_string(&self) -> &str { &self.name }
}

impl Default for StatBasedRollModifier {
    fn default() -> Self {
        StatBasedRollModifier { name: String::new(), value: 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_name_and_value() {
        let m = StatBasedRollModifier::new("Strength Bonus", 2);
        assert_eq!(m.get_report_string(), "Strength Bonus");
        assert_eq!(m.get_modifier(), 2);
    }

    #[test]
    fn is_modifier_included_always_false() {
        assert!(!StatBasedRollModifier::new("x", 5).is_modifier_included());
    }

    #[test]
    fn default_is_zero_empty() {
        let m = StatBasedRollModifier::default();
        assert_eq!(m.get_modifier(), 0);
        assert!(m.name.is_empty());
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", StatBasedRollModifier::default()).is_empty());
    }


    #[test]
    fn two_modifiers_with_same_fields_are_equal() {
        let m1 = StatBasedRollModifier::new("Str", 3);
        let m2 = StatBasedRollModifier::new("Str", 3);
        assert_eq!(m1, m2);
    }
}
