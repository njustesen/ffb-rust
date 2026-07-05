// client-only: Java Swing dialog UI — headless decisions handled by network_encoder/mod.rs.
pub struct Dialogconfirmendactionhandler;

impl Dialogconfirmendactionhandler {
    pub fn new() -> Self { Self }
}

impl Default for Dialogconfirmendactionhandler {
    fn default() -> Self { Self::new() }
}
