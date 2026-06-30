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
