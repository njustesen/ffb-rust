// client-only: Java Swing dialog UI — headless decisions handled by network_encoder/mod.rs.
pub struct Dropdownpanel;

impl Dropdownpanel {
    pub fn new() -> Self { Self }
}

impl Default for Dropdownpanel {
    fn default() -> Self { Self::new() }
}
