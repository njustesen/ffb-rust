// client-only: Java Swing dialog UI — headless decisions handled by network_encoder/mod.rs.
pub struct Dialogjoinhandler;

impl Dialogjoinhandler {
    pub fn new() -> Self { Self }
}

impl Default for Dialogjoinhandler {
    fn default() -> Self { Self::new() }
}
