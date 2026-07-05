// client-only: Java Swing dialog UI — headless decisions handled by network_encoder/mod.rs.
pub struct Dialoghandler;

impl Dialoghandler {
    pub fn new() -> Self { Self }
}

impl Default for Dialoghandler {
    fn default() -> Self { Self::new() }
}
