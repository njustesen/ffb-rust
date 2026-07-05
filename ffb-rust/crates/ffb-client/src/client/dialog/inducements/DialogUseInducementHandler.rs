// client-only: Java Swing dialog UI — headless decisions handled by network_encoder/mod.rs.
pub struct Dialoguseinducementhandler;

impl Dialoguseinducementhandler {
    pub fn new() -> Self { Self }
}

impl Default for Dialoguseinducementhandler {
    fn default() -> Self { Self::new() }
}
