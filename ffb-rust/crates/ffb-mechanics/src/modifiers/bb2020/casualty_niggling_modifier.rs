pub struct CasualtyNigglingModifier {
    inner: super::casualty_modifier::CasualtyModifier,
}

impl CasualtyNigglingModifier {
    pub fn new(name: impl Into<String>, modifier: i32) -> Self {
        Self { inner: super::casualty_modifier::CasualtyModifier::new(name, modifier) }
    }

    pub fn get_modifier(&self) -> i32 { self.inner.get_modifier() }
    pub fn get_name(&self) -> &str { self.inner.get_name() }
    pub fn report_string(&self) -> String { self.inner.name.clone() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_modifier_returns_value() {
        let m = CasualtyNigglingModifier::new("Niggling Injury", 1);
        assert_eq!(m.get_modifier(), 1);
    }

    #[test]
    fn report_string_returns_name() {
        let m = CasualtyNigglingModifier::new("Niggling Injury", 1);
        assert_eq!(m.report_string(), "Niggling Injury");
    }

    #[test]
    fn get_name_returns_name() {
        let m = CasualtyNigglingModifier::new("Double Niggling", 2);
        assert_eq!(m.get_name(), "Double Niggling");
    }

    #[test]
    fn get_modifier_returns_various_values() {
        assert_eq!(CasualtyNigglingModifier::new("x", 0).get_modifier(), 0);
        assert_eq!(CasualtyNigglingModifier::new("x", 3).get_modifier(), 3);
        assert_eq!(CasualtyNigglingModifier::new("x", -1).get_modifier(), -1);
    }

    #[test]
    fn report_string_matches_get_name() {
        let m = CasualtyNigglingModifier::new("Three Nigglings", 3);
        assert_eq!(m.report_string(), m.get_name());
    }
}
