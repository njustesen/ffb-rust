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
