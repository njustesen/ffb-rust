// client-only: Java Swing dialog UI — headless decisions handled by network_encoder/mod.rs.
pub struct Dialoginterceptionhandler;

impl Dialoginterceptionhandler {
    pub fn new() -> Self { Self }
}

impl Default for Dialoginterceptionhandler {
    fn default() -> Self { Self::new() }
}
