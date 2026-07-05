// client-only: Java Swing dialog UI — headless decisions handled by network_encoder/mod.rs.
pub struct Dialogcoinchoicehandler;

impl Dialogcoinchoicehandler {
    pub fn new() -> Self { Self }
}

impl Default for Dialogcoinchoicehandler {
    fn default() -> Self { Self::new() }
}
