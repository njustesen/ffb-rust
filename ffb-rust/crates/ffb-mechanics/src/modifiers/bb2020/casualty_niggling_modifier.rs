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
}
