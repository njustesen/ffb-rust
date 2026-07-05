// client-only: Java Swing dialog UI — headless decisions handled by network_encoder/mod.rs.
pub struct Dialogabouthandler;

impl Dialogabouthandler {
    pub fn new() -> Self { Self }
}

impl Default for Dialogabouthandler {
    fn default() -> Self { Self::new() }
}
