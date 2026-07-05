// client-only: Java Swing dialog UI — headless decisions handled by network_encoder/mod.rs.
pub struct Dialogplayerchoicehandler;

impl Dialogplayerchoicehandler {
    pub fn new() -> Self { Self }
}

impl Default for Dialogplayerchoicehandler {
    fn default() -> Self { Self::new() }
}
