// client-only: Java Swing dialog UI — headless decisions handled by network_encoder/mod.rs.
pub struct Dialogcredits;

impl Dialogcredits {
    pub fn new() -> Self { Self }
}

impl Default for Dialogcredits {
    fn default() -> Self { Self::new() }
}
