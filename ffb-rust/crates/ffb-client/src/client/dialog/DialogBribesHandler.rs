// client-only: Java Swing dialog UI — headless decisions handled by network_encoder/mod.rs.
pub struct Dialogbribeshandler;

impl Dialogbribeshandler {
    pub fn new() -> Self { Self }
}

impl Default for Dialogbribeshandler {
    fn default() -> Self { Self::new() }
}
