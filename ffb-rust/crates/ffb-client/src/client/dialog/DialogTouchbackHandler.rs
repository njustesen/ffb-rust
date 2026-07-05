// client-only: Java Swing dialog UI — headless decisions handled by network_encoder/mod.rs.
pub struct Dialogtouchbackhandler;

impl Dialogtouchbackhandler {
    pub fn new() -> Self { Self }
}

impl Default for Dialogtouchbackhandler {
    fn default() -> Self { Self::new() }
}
